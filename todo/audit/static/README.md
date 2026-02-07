# Audit Service Web Interface

A modern, responsive web interface for the Audit Service code analysis platform.

## Features

- 🎯 **Interactive Dashboard**: Clean, intuitive interface for running code audits
- 📊 **Visual Analytics**: Charts and graphs showing severity distribution and statistics
- 🤖 **LLM Integration**: Optional AI-powered code analysis with Grok 4.1
- 📋 **Task Generation**: Automatically generates actionable tasks from audit findings
- 🏷️ **Tag Scanner**: Detect and analyze audit annotations in source code
- 📦 **Repository Cloning**: Clone and analyze Git repositories directly
- 💾 **Export Options**: Download reports and tasks as JSON
- 📱 **Responsive Design**: Works on desktop, tablet, and mobile devices

## Usage

### Starting the Server

```bash
# From the audit directory
cargo run --bin audit-server

# Or with custom configuration
RUST_LOG=info cargo run --bin audit-server
```

The web interface will be available at `http://localhost:3000` by default.

### Running an Audit

1. **Enter Repository**: Provide a Git URL or local path
2. **Configure Options**:
   - Branch (optional)
   - Enable LLM analysis for AI-powered insights
   - Include test files
   - Focus areas (security, performance, etc.)
3. **Click "Start Audit"**: The analysis will run and display results

### Quick Actions

- **Tag Scanner**: Scan a directory for audit tags (`@audit-tag`, `@audit-todo`, etc.)
- **Clone Repository**: Clone a Git repository to the workspace
- **Static Analysis**: Run static code analysis without LLM

## API Endpoints

The web interface communicates with the following API endpoints:

### Health Check
```
GET /health
```

### Create Audit
```
POST /api/audit
Content-Type: application/json

{
  "repository": "https://github.com/user/repo",
  "branch": "main",
  "enable_llm": true,
  "include_tests": false,
  "focus": ["security", "performance"]
}
```

### Get Audit Report
```
GET /api/audit/:id
```

### Get Audit Tasks
```
GET /api/audit/:id/tasks
```

### Clone Repository
```
POST /api/clone
Content-Type: application/json

{
  "url": "https://github.com/user/repo",
  "branch": "main"
}
```

### Scan Tags
```
POST /api/scan/tags
Content-Type: application/json

{
  "path": "/path/to/directory"
}
```

### Static Analysis
```
POST /api/scan/static
Content-Type: application/json

{
  "path": "/path/to/directory"
}
```

## File Structure

```
static/
├── index.html          # Main HTML file
├── css/
│   └── styles.css      # Styling and responsive design
├── js/
│   └── app.js          # Application logic and API calls
└── README.md           # This file
```

## Customization

### Styling

Edit `css/styles.css` to customize colors, fonts, and layout:

```css
:root {
    --primary-color: #2563eb;
    --primary-hover: #1d4ed8;
    /* ... other variables */
}
```

### API Configuration

By default, the app uses relative URLs (`/api/*`). To use a different API server, modify the `API_BASE` constant in `js/app.js`:

```javascript
const API_BASE = 'http://localhost:3000/api';
```

## Browser Compatibility

- Chrome/Edge 90+
- Firefox 88+
- Safari 14+
- Opera 76+

## Development

### Adding New Features

1. Add HTML markup to `index.html`
2. Add styles to `css/styles.css`
3. Add functionality to `js/app.js`

### Debugging

Open browser DevTools (F12) to:
- View console logs
- Inspect network requests
- Debug JavaScript

## Security Considerations

- The web interface should be run behind a reverse proxy (nginx, Caddy) in production
- Configure CORS appropriately for your deployment
- Use HTTPS in production environments
- Sanitize all user inputs (implemented via `escapeHtml()` function)

## Screenshots

### Main Dashboard
The main dashboard provides a clean interface for configuring and running audits.

### Results View
After running an audit, view detailed statistics, severity charts, critical files, and generated tasks.

### Quick Actions
Use modals for quick operations like tag scanning and repository cloning.

## License

Same as the parent Audit Service project.

## Support

For issues or questions, please open an issue on the GitHub repository.