// ===== API Base URL =====
const API_BASE = '/api';

// ===== State Management =====
let currentReport = null;

// ===== Initialization =====
document.addEventListener('DOMContentLoaded', () => {
    checkServerHealth();
    setupEventListeners();
});

// ===== Event Listeners =====
function setupEventListeners() {
    // Main audit form
    document.getElementById('audit-form').addEventListener('submit', handleAuditSubmit);

    // Tag scanner form
    document.getElementById('tag-scanner-form').addEventListener('submit', handleTagScan);

    // Clone repository form
    document.getElementById('clone-form').addEventListener('submit', handleCloneRepo);

    // Static analysis form
    document.getElementById('static-form').addEventListener('submit', handleStaticAnalysis);

    // Download buttons
    document.getElementById('download-json').addEventListener('click', downloadJSON);
    document.getElementById('download-tasks').addEventListener('click', downloadTasks);
}

// ===== Server Health Check =====
async function checkServerHealth() {
    try {
        const response = await fetch('/health');
        const data = await response.json();

        if (data.status === 'healthy') {
            updateHealthStatus(true, data.version);
        } else {
            updateHealthStatus(false, 'Unknown');
        }
    } catch (error) {
        updateHealthStatus(false, 'Offline');
    }
}

function updateHealthStatus(healthy, version) {
    const indicator = document.querySelector('.status-indicator');
    const statusText = document.querySelector('.status-text');
    const versionEl = document.getElementById('version');

    if (healthy) {
        indicator.classList.add('healthy');
        statusText.textContent = 'Online';
        if (versionEl && version !== 'Unknown') {
            versionEl.textContent = version;
        }
    } else {
        indicator.classList.remove('healthy');
        statusText.textContent = 'Offline';
    }
}

// ===== Main Audit Form Handler =====
async function handleAuditSubmit(e) {
    e.preventDefault();

    const submitBtn = document.getElementById('submit-btn');
    const loading = document.getElementById('loading');
    const resultsSection = document.getElementById('results-section');

    // Gather form data
    const formData = {
        repository: document.getElementById('repository').value,
        branch: document.getElementById('branch').value || null,
        enable_llm: document.getElementById('enable-llm').checked,
        include_tests: document.getElementById('include-tests').checked,
        focus: document.getElementById('focus').value
            ? document.getElementById('focus').value.split(',').map(s => s.trim())
            : []
    };

    // Show loading
    submitBtn.disabled = true;
    submitBtn.textContent = 'Running...';
    loading.classList.remove('hidden');
    resultsSection.classList.add('hidden');

    try {
        const response = await fetch(`${API_BASE}/audit`, {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json'
            },
            body: JSON.stringify(formData)
        });

        if (!response.ok) {
            const error = await response.json();
            throw new Error(error.error || 'Audit failed');
        }

        const result = await response.json();
        currentReport = result.report;

        // Display results
        displayAuditResults(result);

    } catch (error) {
        showError(error.message);
    } finally {
        submitBtn.disabled = false;
        submitBtn.textContent = 'Start Audit';
        loading.classList.add('hidden');
    }
}

// ===== Display Audit Results =====
function displayAuditResults(data) {
    const report = data.report;
    const resultsSection = document.getElementById('results-section');

    // Update stats
    document.getElementById('total-files').textContent = report.summary.total_files;
    document.getElementById('total-issues').textContent = report.summary.total_issues;
    document.getElementById('critical-files').textContent = report.summary.critical_files;
    document.getElementById('total-tasks').textContent = report.summary.total_tasks;

    // Update audit ID
    document.getElementById('audit-id').textContent = report.id;

    // Render severity chart
    renderSeverityChart(report.issues_by_severity);

    // Render critical files
    renderCriticalFiles(report.files);

    // Render tasks
    renderTasks(report.tasks);

    // Render tags summary
    renderTagsSummary(report.files);

    // Show results
    resultsSection.classList.remove('hidden');

    // Scroll to results
    resultsSection.scrollIntoView({ behavior: 'smooth', block: 'start' });
}

// ===== Render Severity Chart =====
function renderSeverityChart(issuesBySeverity) {
    const container = document.getElementById('severity-chart');
    container.innerHTML = '';

    const severities = ['Critical', 'High', 'Medium', 'Low'];
    const total = Object.values(issuesBySeverity).reduce((sum, count) => sum + count, 0);

    severities.forEach(severity => {
        const count = issuesBySeverity[severity] || 0;
        const percentage = total > 0 ? (count / total * 100) : 0;

        const item = document.createElement('div');
        item.className = 'severity-item';

        item.innerHTML = `
            <div class="severity-label ${severity.toLowerCase()}">${severity}</div>
            <div class="severity-bar-container">
                <div class="severity-bar ${severity.toLowerCase()}" style="width: ${percentage}%">
                    ${count > 0 ? count : ''}
                </div>
            </div>
        `;

        container.appendChild(item);
    });
}

// ===== Render Critical Files =====
function renderCriticalFiles(files) {
    const container = document.getElementById('critical-files-list');
    container.innerHTML = '';

    // Filter and sort critical/high priority files
    const criticalFiles = files
        .filter(f => f.priority === 'Critical' || f.priority === 'High')
        .sort((a, b) => {
            const priorities = { Critical: 4, High: 3, Medium: 2, Low: 1 };
            return priorities[b.priority] - priorities[a.priority];
        })
        .slice(0, 10); // Show top 10

    if (criticalFiles.length === 0) {
        container.innerHTML = '<p style="color: var(--text-secondary);">No critical files found. Great job! 🎉</p>';
        return;
    }

    criticalFiles.forEach(file => {
        const item = document.createElement('div');
        item.className = `file-item ${file.priority.toLowerCase()}`;

        const pathDisplay = file.path.length > 60
            ? '...' + file.path.slice(-60)
            : file.path;

        item.innerHTML = `
            <div class="file-path">${escapeHtml(pathDisplay)}</div>
            <div class="file-meta">
                <span class="file-badge ${file.priority.toLowerCase()}">${file.priority}</span>
                <span>${file.category || 'Unknown'}</span>
                <span>${file.issues.length} issue(s)</span>
                ${file.llm_analysis ? '<span>🤖 AI Analyzed</span>' : ''}
            </div>
        `;

        container.appendChild(item);
    });
}

// ===== Render Tasks =====
function renderTasks(tasks) {
    const container = document.getElementById('tasks-list');
    container.innerHTML = '';

    if (tasks.length === 0) {
        container.innerHTML = '<p style="color: var(--text-secondary);">No tasks generated.</p>';
        return;
    }

    // Show first 20 tasks
    const displayTasks = tasks.slice(0, 20);

    displayTasks.forEach(task => {
        const item = document.createElement('div');
        item.className = `task-item ${task.priority.toLowerCase()}-priority`;

        item.innerHTML = `
            <div class="task-header">
                <div class="task-title">${escapeHtml(task.title)}</div>
                <span class="task-priority ${task.priority.toLowerCase()}">${task.priority}</span>
            </div>
            <div class="task-description">${escapeHtml(task.description)}</div>
            ${task.file ? `<div class="task-file">📄 ${escapeHtml(task.file)}</div>` : ''}
        `;

        container.appendChild(item);
    });

    if (tasks.length > 20) {
        const more = document.createElement('p');
        more.style.color = 'var(--text-secondary)';
        more.style.marginTop = '10px';
        more.textContent = `+ ${tasks.length - 20} more tasks...`;
        container.appendChild(more);
    }
}

// ===== Render Tags Summary =====
function renderTagsSummary(files) {
    const container = document.getElementById('tags-summary');
    container.innerHTML = '';

    // Count tags by type
    const tagCounts = {};
    files.forEach(file => {
        file.tags.forEach(tag => {
            const tagType = tag.tag_type;
            tagCounts[tagType] = (tagCounts[tagType] || 0) + 1;
        });
    });

    if (Object.keys(tagCounts).length === 0) {
        container.innerHTML = '<p style="color: var(--text-secondary);">No audit tags found.</p>';
        return;
    }

    Object.entries(tagCounts).forEach(([tagType, count]) => {
        const item = document.createElement('div');
        item.className = 'tag-count';

        item.innerHTML = `
            <div class="tag-count-value">${count}</div>
            <div class="tag-count-label">${tagType}</div>
        `;

        container.appendChild(item);
    });
}

// ===== Tag Scanner =====
function showTagScanner() {
    document.getElementById('tag-scanner-modal').classList.remove('hidden');
    document.getElementById('tag-scanner-modal').classList.add('show');
}

async function handleTagScan(e) {
    e.preventDefault();

    const path = document.getElementById('tag-path').value;
    const resultsDiv = document.getElementById('tag-results');

    resultsDiv.innerHTML = '<div class="spinner"></div>';
    resultsDiv.classList.remove('hidden');

    try {
        const response = await fetch(`${API_BASE}/scan/tags`, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ path })
        });

        if (!response.ok) {
            const error = await response.json();
            throw new Error(error.error || 'Tag scan failed');
        }

        const data = await response.json();

        let html = `<h3>Found ${data.total} tags</h3>`;
        html += '<div style="margin-top: 15px;">';

        Object.entries(data.by_type).forEach(([type, count]) => {
            html += `<div style="margin: 5px 0;"><strong>${type}:</strong> ${count}</div>`;
        });

        html += '</div>';
        resultsDiv.innerHTML = html;

    } catch (error) {
        resultsDiv.innerHTML = `<div style="color: var(--danger-color);">Error: ${error.message}</div>`;
    }
}

// ===== Clone Repository =====
function showCloneRepo() {
    document.getElementById('clone-modal').classList.remove('hidden');
    document.getElementById('clone-modal').classList.add('show');
}

async function handleCloneRepo(e) {
    e.preventDefault();

    const url = document.getElementById('clone-url').value;
    const branch = document.getElementById('clone-branch').value || null;
    const resultsDiv = document.getElementById('clone-results');

    resultsDiv.innerHTML = '<div class="spinner"></div>';
    resultsDiv.classList.remove('hidden');

    try {
        const response = await fetch(`${API_BASE}/clone`, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ url, branch })
        });

        if (!response.ok) {
            const error = await response.json();
            throw new Error(error.error || 'Clone failed');
        }

        const data = await response.json();

        resultsDiv.innerHTML = `
            <h3>Repository Cloned Successfully</h3>
            <div style="margin-top: 15px;">
                <div><strong>Path:</strong> ${escapeHtml(data.path)}</div>
                <div><strong>Branch:</strong> ${escapeHtml(data.branch)}</div>
                <div><strong>Commits:</strong> ${data.commit_count}</div>
            </div>
        `;

    } catch (error) {
        resultsDiv.innerHTML = `<div style="color: var(--danger-color);">Error: ${error.message}</div>`;
    }
}

// ===== Static Analysis =====
function showStaticAnalysis() {
    document.getElementById('static-modal').classList.remove('hidden');
    document.getElementById('static-modal').classList.add('show');
}

async function handleStaticAnalysis(e) {
    e.preventDefault();

    const path = document.getElementById('static-path').value;
    const resultsDiv = document.getElementById('static-results');

    resultsDiv.innerHTML = '<div class="spinner"></div>';
    resultsDiv.classList.remove('hidden');

    try {
        const response = await fetch(`${API_BASE}/scan/static`, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ path })
        });

        if (!response.ok) {
            const error = await response.json();
            throw new Error(error.error || 'Static analysis failed');
        }

        const data = await response.json();

        resultsDiv.innerHTML = `
            <h3>Static Analysis Complete</h3>
            <div style="margin-top: 15px;">
                <div><strong>Total Files:</strong> ${data.total_files}</div>
                <div><strong>Total Issues:</strong> ${data.total_issues}</div>
                <div><strong>Critical Files:</strong> ${data.critical_files}</div>
            </div>
        `;

    } catch (error) {
        resultsDiv.innerHTML = `<div style="color: var(--danger-color);">Error: ${error.message}</div>`;
    }
}

// ===== Modal Controls =====
function closeModal(modalId) {
    const modal = document.getElementById(modalId);
    modal.classList.remove('show');
    modal.classList.add('hidden');
}

// ===== Download Functions =====
function downloadJSON() {
    if (!currentReport) {
        showError('No report available to download');
        return;
    }

    const dataStr = JSON.stringify(currentReport, null, 2);
    const blob = new Blob([dataStr], { type: 'application/json' });
    const url = URL.createObjectURL(blob);
    const link = document.createElement('a');
    link.href = url;
    link.download = `audit-report-${currentReport.id}.json`;
    link.click();
    URL.revokeObjectURL(url);
}

function downloadTasks() {
    if (!currentReport || !currentReport.tasks) {
        showError('No tasks available to download');
        return;
    }

    const dataStr = JSON.stringify(currentReport.tasks, null, 2);
    const blob = new Blob([dataStr], { type: 'application/json' });
    const url = URL.createObjectURL(blob);
    const link = document.createElement('a');
    link.href = url;
    link.download = `audit-tasks-${currentReport.id}.json`;
    link.click();
    URL.revokeObjectURL(url);
}

// ===== Error Handling =====
function showError(message) {
    const errorDisplay = document.getElementById('error-display');
    const errorContent = errorDisplay.querySelector('.error-content');

    errorContent.textContent = message;
    errorDisplay.classList.remove('hidden');

    // Auto-hide after 5 seconds
    setTimeout(() => {
        errorDisplay.classList.add('hidden');
    }, 5000);
}

function closeError() {
    document.getElementById('error-display').classList.add('hidden');
}

// ===== Utility Functions =====
function escapeHtml(text) {
    const div = document.createElement('div');
    div.textContent = text;
    return div.innerHTML;
}

// ===== Close modals on outside click =====
window.onclick = function(event) {
    if (event.target.classList.contains('modal')) {
        event.target.classList.remove('show');
        event.target.classList.add('hidden');
    }
};
