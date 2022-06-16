CREATE TABLE installed_package (
    name TEXT PRIMARY KEY,
    installed_version TEXT,
    requested_version TEXT
) STRICT;

CREATE TABLE installed_file (
    path TEXT PRIMARY KEY,
    package_name TEXT REFERENCES installed_package(name) ON DELETE CASCADE
) STRICT;
