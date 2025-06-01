# Clicking Circles Web Player

A web-based version of the clicking-circles-player, designed for mobile usage.

## Features

- Play osu! songs with background images
- Mobile-friendly interface
- Search functionality
- Unicode/non-Unicode toggle
- Progress bar with seeking
- Background image display

## Setup

### Backend (Python/FastAPI)

1. Navigate to the backend directory:
```bash
cd backend
```

2. Install dependencies:
```bash
pip install -r requirements.txt
```

3. Run the server:
```bash
uvicorn main:app --reload --host 0.0.0.0 --port 8000
```

### Frontend (Next.js)

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

## Usage

1. Make sure you have a `song.json` file in the root directory (same format as the original clicking-circles-player)
2. Start both the backend and frontend servers
3. Access the web player at `http://localhost:3000`

## Development

- Backend: Python FastAPI
- Frontend: Next.js + React + TypeScript
- Styling: Tailwind CSS
- Audio: Howler.js
- Icons: React Icons 