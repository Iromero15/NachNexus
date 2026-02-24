-- Add migration script here
-- 1. Creamos los tipos ENUM restrictivos y optimizados
CREATE TYPE user_role AS ENUM ('admin', 'commonUser');
CREATE TYPE media_category AS ENUM ('movie', 'series', 'music', 'audiobook');
CREATE TYPE request_status AS ENUM ('pending', 'approved', 'rejected');

-- 2. Tabla de usuarios usando el ENUM para el rol
CREATE TABLE IF NOT EXISTS users (
    id SERIAL PRIMARY KEY,
    username VARCHAR(255) UNIQUE NOT NULL,
    role user_role NOT NULL DEFAULT 'commonUser',
    jellyfin_id VARCHAR(255),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- 3. Tabla de pedidos usando los ENUMs y el JSONB
CREATE TABLE IF NOT EXISTS media_requests (
    id SERIAL PRIMARY KEY,
    user_id INTEGER NOT NULL REFERENCES users(id),
    media_type media_category NOT NULL, 
    status request_status NOT NULL DEFAULT 'pending',
    file_path TEXT NOT NULL, 
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb, 
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);