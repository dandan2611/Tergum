# Tergum
Easily store backups of your projects on a S3 compatible server.

## Usage

### Environment

Copy the provided `.env.example` file to `.env` and fill in the required values.

### Usage

Configure which files and folders you want to backup in the `.backupsrc` file.
```
src/
data/
```

You can also ignore files and folders by adding them to the `.backupignore` file.
```
.git/
node_modules/
```

Each subfolder saved in the backup can contain its own `.backupignore` file.

Use `tergum -h` to see all available commands.