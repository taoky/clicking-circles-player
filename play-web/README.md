# Clicking Circles Web Player

A web-based version of the clicking-circles-player, designed for mobile usage.

## Features

- Play osu! songs with background images
- Mobile-friendly interface
- Search functionality
- Unicode/non-Unicode toggle
- Progress bar with seeking
- Background image display
- Docker deployment support

## Setup

There are three ways to run this application:

### 1. Using Docker (Recommended)

1. Make sure you have Docker and Docker Compose installed
2. Clone this repository
3. Update the volumes in `docker-compose.yml` to point to your osu! files directory and song.json:
```yaml
volumes:
  - /path/to/osu/files:/data/files:ro
  - /path/to/song.json:/data/song.json:ro
```
4. Run the application:
```bash
docker compose up -d
```
5. Access the web player at `http://localhost:8000`

### 2. Development Setup (Frontend Only)

1. Navigate to the frontend directory:
```bash
cd frontend
```

2. Install dependencies:
```bash
pnpm install
```

3. Run the development server:
```bash
pnpm dev
```

4. Access the development server at `http://localhost:3000`

### 3. Production Build (Frontend)

1. Navigate to the frontend directory:
```bash
cd frontend
```

2. Install dependencies:
```bash
pnpm install
```

3. Build the application:
```bash
pnpm build
```

4. The output will be in the `frontend/out` directory, which can be served using any static file server

## Prerequisites

- Node.js 18+ (for development)
- pnpm (for package management)
- Docker and Docker Compose (for containerized deployment)
- osu! game files and a valid song.json file

## Project Structure

- `frontend/`: Next.js frontend application
  - `src/`: Source code
  - `public/`: Static assets
- `nginx.conf`: Nginx configuration for production deployment
- `docker-compose.yml`: Docker Compose configuration
- `Dockerfile`: Docker build configuration

## Technologies Used

- Frontend:
  - Next.js
  - React
  - TypeScript
  - Tailwind CSS
  - Howler.js (audio playback)
  - React Icons
- Deployment:
  - Docker
  - Nginx

## Contributing

Feel free to open issues or submit pull requests for any bugs or improvements.

## License

[Add your license information here] 