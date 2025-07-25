CREATE TABLE IF NOT EXISTS photos (
    id UUID PRIMARY KEY,
    caption TEXT,
    url TEXT,
    event_id UUID REFERENCES events(id) ON DELETE CASCADE,
    posted_by UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL,
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_photos_posted_by ON photos(posted_by);
CREATE INDEX IF NOT EXISTS idx_photos_caption ON photos(caption);
CREATE INDEX IF NOT EXISTS idx_photos_event_id ON photos(event_id);
CREATE INDEX IF NOT EXISTS idx_photos_created_at ON photos(created_at);
