CREATE TABLE colors (
    id SERIAL PRIMARY KEY,
    r SMALLINT NOT NULL,
    g SMALLINT NOT NULL,
    b SMALLINT NOT NULL
);

CREATE TABLE users (
    id SERIAL PRIMARY KEY,
    username VARCHAR(255) NOT NULL,
    password VARCHAR(255) NOT NULL,
    color_id INT,
    FOREIGN KEY (color_id) REFERENCES colors(id)
);

CREATE TABLE messages (
    id SERIAL PRIMARY KEY,
    user_id INT NOT NULL,
    text TEXT NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP NOT NULL,
    FOREIGN KEY (user_id) REFERENCES users(id)
);
