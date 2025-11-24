# RustyShort

### Prerequisites

- Docker and Docker Compose (recommended)
- OR Rust 1.75+ and PostgreSQL 14+ (for local development)

### Local Development

1. Clone the repository and navigate to the project directory

2. Copy the environment file (create `.env` based on the example):
```bash
DATABASE_URL=postgres://rustyshort:rustyshort@localhost:5432/rustyshort
SERVER_HOST=0.0.0.0
SERVER_PORT=8080
BASE_URL=http://localhost:8080
LOG_LEVEL=info
RUST_LOG=rustyshort=debug,tower_http=debug
```

3. Start PostgreSQL (or use Docker):
```bash
docker run -d \
  --name rustyshort-postgres \
  -e POSTGRES_USER=rustyshort \
  -e POSTGRES_PASSWORD=rustyshort \
  -e POSTGRES_DB=rustyshort \
  -p 5432:5432 \
  postgres:16-alpine
```

4. Run the application:
```bash
cargo run
```

### Docker Deployment

```bash
docker-compose up -d
```

The application will be available at:
- **http://localhost** - Main application (via nginx with rate limiting)
- **http://localhost/health** - Health check
- **http://localhost/metrics** - Prometheus metrics (restricted to private IPs)


## API Endpoints

### Create Short Link
```bash
POST /api/v1/links
Content-Type: application/json

{
  "url": "https://www.example.com/very/long/path",
  "custom_alias": "my-link",
  "expires_in": 3600
}

Response:
{
  "key": "my-link",
  "short_url": "http://localhost:8080/my-link",
  "original_url": "https://www.example.com/very/long/path",
  "qr_code_url": "http://localhost:8080/qr/my-link",
  "created_at": "2025-11-24T10:00:00Z",
  "expires_at": "2025-11-24T11:00:00Z"
}
```

### Redirect to Original URL
```bash
GET /{key}
```

### Get Link Statistics
```bash
GET /api/v1/links/{key}/stats

Response:
{
  "key": "my-link",
  "original_url": "https://www.example.com/very/long/path",
  "click_count": 42,
  "created_at": "2025-11-24T10:00:00Z",
  "expires_at": "2025-11-24T11:00:00Z"
}
```

### Generate QR Code
```bash
GET /qr/{key}
```

### Delete Link
```bash
DELETE /api/v1/links/{key}
```

### List Links
```bash
GET /api/v1/links?limit=50&offset=0
```

### Health Check
```bash
GET /health
```

### Metrics
```bash
GET /metrics
```

## Configuration

All configuration is done via environment variables:

| Variable | Description | Default |
|----------|-------------|---------|
| `DATABASE_URL` | PostgreSQL connection string | Required |
| `SERVER_HOST` | Server bind address | `0.0.0.0` |
| `SERVER_PORT` | Server port | `8080` |
| `BASE_URL` | Base URL for short links | `http://localhost:8080` |
| `CACHE_TTL` | Cache TTL in seconds | `3600` |
| `CACHE_MAX_CAPACITY` | Maximum cache entries | `10000` |
| `RATE_LIMIT_PER_SECOND` | Rate limit per second | `10` |
| `RATE_LIMIT_BURST_SIZE` | Rate limit burst size | `50` |
| `DEFAULT_REDIRECT_TYPE` | HTTP redirect status code | `301` |
| `RUST_LOG` | Logging level | `info` |


## License

MIT

## Contributing

Contributions are welcome! Please open an issue or submit a pull request.
