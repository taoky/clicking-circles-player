from fastapi import FastAPI, HTTPException, Header
from fastapi.staticfiles import StaticFiles
from fastapi.middleware.cors import CORSMiddleware
from fastapi.responses import FileResponse, StreamingResponse
from pydantic import BaseModel, Field
from typing import List, Optional
import json
from pathlib import Path
import aiofiles
import magic

app = FastAPI()

# Enable CORS for development
app.add_middleware(
    CORSMiddleware,
    allow_origins=["*"],  # In production, replace with your frontend URL
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)


class Song(BaseModel):
    Title: str
    TitleUnicode: str = ""
    Artist: str
    ArtistUnicode: str = ""
    Source: str
    Tags: List[str]
    AudioHash: str
    BGHashes: List[str]

    def __init__(self, **data):
        super().__init__(**data)
        if not self.TitleUnicode:
            self.TitleUnicode = self.Title
        if not self.ArtistUnicode:
            self.ArtistUnicode = self.Artist


# Global variables to store configuration
songs_data: List[Song] = []
osu_files_path: Path = None


@app.on_event("startup")
async def startup_event():
    global songs_data, osu_files_path
    # These paths should be configurable via environment variables in production
    with open("/data/song.json") as f:
        songs_data = [Song.model_validate(item) for item in json.load(f)]
    osu_files_path = Path("/data/files").expanduser()


@app.get("/api/songs")
async def list_songs():
    return songs_data


@app.get("/api/songs/search")
async def search_songs(q: str):
    q = q.lower()
    results = []
    for idx, song in enumerate(songs_data):
        if (
            q in song.Title.lower()
            or q in song.Artist.lower()
            or q in song.Source.lower()
            or q in song.TitleUnicode.lower()
            or q in song.ArtistUnicode.lower()
            or q in " ".join(song.Tags).lower()
        ):
            results.append({"index": idx, "song": song})
    return results


def get_file_path(hash: str) -> Path:
    return osu_files_path / hash[0] / hash[:2] / hash


@app.get("/api/audio/{hash}")
async def get_audio(hash: str, range: str = Header(None)):
    try:
        file_path = get_file_path(hash)
        if not file_path.exists():
            raise HTTPException(status_code=404, detail="Audio file not found")
        
        # Detect file type using python-magic
        mime = magic.Magic(mime=True)
        file_mime = mime.from_file(str(file_path))
        
        file_size = file_path.stat().st_size
        
        # Handle range request
        if range is not None:
            try:
                start_b, end_b = range.replace("bytes=", "").split("-")
                start = int(start_b)
                end = int(end_b) if end_b else file_size - 1
                if end >= file_size:
                    end = file_size - 1
                content_length = end - start + 1

                headers = {
                    "Content-Range": f"bytes {start}-{end}/{file_size}",
                    "Accept-Ranges": "bytes",
                    "Content-Length": str(content_length),
                    "Content-Type": file_mime
                }

                async def stream_range():
                    async with aiofiles.open(file_path, mode='rb') as f:
                        await f.seek(start)
                        chunk_size = 8192  # 8KB chunks
                        remaining = content_length
                        while remaining > 0:
                            chunk = await f.read(min(chunk_size, remaining))
                            if not chunk:
                                break
                            remaining -= len(chunk)
                            yield chunk

                return StreamingResponse(
                    stream_range(),
                    headers=headers,
                    status_code=206
                )
            except ValueError:
                # If range header is malformed, fall back to sending entire file
                pass
        
        # If no range header or parsing failed, send entire file
        return FileResponse(
            file_path,
            media_type=file_mime,
            headers={"Accept-Ranges": "bytes"}
        )
    except Exception as e:
        raise HTTPException(status_code=500, detail=str(e))


@app.get("/api/image/{hash}")
async def get_image(hash: str):
    try:
        file_path = get_file_path(hash)
        if not file_path.exists():
            raise HTTPException(status_code=404, detail="Image file not found")
        return FileResponse(file_path, media_type="image/jpeg")
    except Exception as e:
        raise HTTPException(status_code=500, detail=str(e))


if __name__ == "__main__":
    import uvicorn

    uvicorn.run(app, host="0.0.0.0", port=8000)
