CREATE TABLE IF NOT EXISTS contributions (
    id UUID PRIMARY KEY,
    title TEXT NOT NULL,
    description TEXT,
    amount DECIMAL(20,9),
    due_date DATE NOT NULL,
    created_by UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL,
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_contributions_created_by ON contributions(created_by);
CREATE INDEX IF NOT EXISTS idx_contributions_title ON contributions(title);
CREATE INDEX IF NOT EXISTS idx_contributions_due_date ON contributions(due_date);
CREATE INDEX IF NOT EXISTS idx_contributions_created_at ON contributions(created_at);