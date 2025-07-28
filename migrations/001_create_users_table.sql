CREATE TYPE user_roles AS ENUM ('superadmin', 'admin', 'member', 'treasurer');

CREATE TABLE IF NOT EXISTS users (
    id UUID PRIMARY KEY,
    fullname VARCHAR(255) NOT NULL,
    email VARCHAR(255) UNIQUE NOT NULL,
    password_hash VARCHAR(255) NOT NULL,
    phone VARCHAR(25),
    user_role user_roles NOT NULL DEFAULT 'member',
    dob DATE,
    photo_url TEXT,
    email_verification_code VARCHAR(255),
    email_verification_expires_at TIMESTAMP WITH TIME ZONE,
    is_email_verified BOOLEAN NOT NULL DEFAULT false,
    is_active BOOLEAN NOT NULL DEFAULT false,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL,
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL
                             );

CREATE INDEX IF NOT EXISTS idx_users_fullname ON users(fullname);
CREATE INDEX IF NOT EXISTS idx_users_email ON users(email);
CREATE INDEX IF NOT EXISTS idx_users_is_active ON users(is_active);
CREATE INDEX IF NOT EXISTS idx_users_user_role ON users(user_role);
CREATE INDEX IF NOT EXISTS idx_users_created_at ON users(created_at);
CREATE INDEX IF NOT EXISTS idx_users_updated_at ON users(updated_at);
