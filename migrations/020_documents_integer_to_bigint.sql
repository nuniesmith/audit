-- Migration: 020_documents_integer_to_bigint.sql
--
-- Promotes INTEGER (INT4) columns to BIGINT (INT8) in the documents family of
-- tables so they match the i64 Rust struct fields in Document, DocumentChunk,
-- and DocumentEmbedding defined in src/db/core.rs.
--
-- Root cause: migration 006_documents.sql created these columns as INTEGER.
-- Every read path in src/db/documents.rs decodes them as i64 / Option<i64>,
-- which sqlx rejects at runtime with:
--   "mismatched types; Rust type i64 (INT8) is not compatible with SQL type INT4"
--
-- All ALTER statements use USING col::BIGINT so the cast is explicit and safe.
-- The IF EXISTS guard on each column means the migration is fully idempotent —
-- running it on a database that already has BIGINT columns is a no-op.

-- ============================================================================
-- documents
-- ============================================================================

DO $$
BEGIN
    IF EXISTS (
        SELECT 1 FROM information_schema.columns
        WHERE table_schema = 'public'
          AND table_name   = 'documents'
          AND column_name  = 'word_count'
          AND data_type    = 'integer'
    ) THEN
        ALTER TABLE documents
            ALTER COLUMN word_count TYPE BIGINT USING word_count::BIGINT;
    END IF;

    IF EXISTS (
        SELECT 1 FROM information_schema.columns
        WHERE table_schema = 'public'
          AND table_name   = 'documents'
          AND column_name  = 'char_count'
          AND data_type    = 'integer'
    ) THEN
        ALTER TABLE documents
            ALTER COLUMN char_count TYPE BIGINT USING char_count::BIGINT;
    END IF;
END
$$;

-- ============================================================================
-- document_chunks
-- ============================================================================

DO $$
BEGIN
    IF EXISTS (
        SELECT 1 FROM information_schema.columns
        WHERE table_schema = 'public'
          AND table_name   = 'document_chunks'
          AND column_name  = 'chunk_index'
          AND data_type    = 'integer'
    ) THEN
        ALTER TABLE document_chunks
            ALTER COLUMN chunk_index TYPE BIGINT USING chunk_index::BIGINT;
    END IF;

    IF EXISTS (
        SELECT 1 FROM information_schema.columns
        WHERE table_schema = 'public'
          AND table_name   = 'document_chunks'
          AND column_name  = 'char_start'
          AND data_type    = 'integer'
    ) THEN
        ALTER TABLE document_chunks
            ALTER COLUMN char_start TYPE BIGINT USING char_start::BIGINT;
    END IF;

    IF EXISTS (
        SELECT 1 FROM information_schema.columns
        WHERE table_schema = 'public'
          AND table_name   = 'document_chunks'
          AND column_name  = 'char_end'
          AND data_type    = 'integer'
    ) THEN
        ALTER TABLE document_chunks
            ALTER COLUMN char_end TYPE BIGINT USING char_end::BIGINT;
    END IF;

    IF EXISTS (
        SELECT 1 FROM information_schema.columns
        WHERE table_schema = 'public'
          AND table_name   = 'document_chunks'
          AND column_name  = 'word_count'
          AND data_type    = 'integer'
    ) THEN
        ALTER TABLE document_chunks
            ALTER COLUMN word_count TYPE BIGINT USING word_count::BIGINT;
    END IF;
END
$$;

-- ============================================================================
-- document_embeddings
-- ============================================================================

DO $$
BEGIN
    IF EXISTS (
        SELECT 1 FROM information_schema.columns
        WHERE table_schema = 'public'
          AND table_name   = 'document_embeddings'
          AND column_name  = 'dimension'
          AND data_type    = 'integer'
    ) THEN
        ALTER TABLE document_embeddings
            ALTER COLUMN dimension TYPE BIGINT USING dimension::BIGINT;
    END IF;
END
$$;

-- ============================================================================
-- scan_checkpoints  (created by auto_scanner.rs / migration 010)
-- ============================================================================
-- auto_scanner.rs decodes last_completed_index, files_analyzed, files_cached,
-- and total_files as i64, but migration 010 defined them as INTEGER (INT4).

DO $$
BEGIN
    IF EXISTS (
        SELECT 1 FROM information_schema.columns
        WHERE table_schema = 'public'
          AND table_name   = 'scan_checkpoints'
          AND column_name  = 'last_completed_index'
          AND data_type    = 'integer'
    ) THEN
        ALTER TABLE scan_checkpoints
            ALTER COLUMN last_completed_index TYPE BIGINT
                USING last_completed_index::BIGINT;
    END IF;

    IF EXISTS (
        SELECT 1 FROM information_schema.columns
        WHERE table_schema = 'public'
          AND table_name   = 'scan_checkpoints'
          AND column_name  = 'files_analyzed'
          AND data_type    = 'integer'
    ) THEN
        ALTER TABLE scan_checkpoints
            ALTER COLUMN files_analyzed TYPE BIGINT
                USING files_analyzed::BIGINT;
    END IF;

    IF EXISTS (
        SELECT 1 FROM information_schema.columns
        WHERE table_schema = 'public'
          AND table_name   = 'scan_checkpoints'
          AND column_name  = 'files_cached'
          AND data_type    = 'integer'
    ) THEN
        ALTER TABLE scan_checkpoints
            ALTER COLUMN files_cached TYPE BIGINT
                USING files_cached::BIGINT;
    END IF;

    IF EXISTS (
        SELECT 1 FROM information_schema.columns
        WHERE table_schema = 'public'
          AND table_name   = 'scan_checkpoints'
          AND column_name  = 'total_files'
          AND data_type    = 'integer'
    ) THEN
        ALTER TABLE scan_checkpoints
            ALTER COLUMN total_files TYPE BIGINT
                USING total_files::BIGINT;
    END IF;
END
$$;

-- ============================================================================
-- Migration complete
-- ============================================================================
