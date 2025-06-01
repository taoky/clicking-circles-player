#!/usr/bin/env python3
# This reads song.json, copys audio file out, add proper extension, and
# adds metadata and covers to them.

import argparse
from pathlib import Path
import json
import subprocess
from tqdm import tqdm

type_mapping = {
    "audio/mpeg": ".mp3",
    "audio/ogg": ".ogg",
    # fallback
    "application/octet-stream": ".mp3",
}

def tell_file_type(music: Path):
    output = subprocess.run(["file", "--brief", "--mime-type", str(music)], capture_output=True, text=True)
    return type_mapping[output.stdout.strip()]

def main(args):
    with open(args.json) as f:
        songs = json.load(f)
    
    def get_path_from_hash(hash):
        return args.data / hash[0] / hash[:2] / hash
    
    for song in tqdm(songs):
        audio_hash = song["AudioHash"]
        background_hashes = song["BGHashes"]
        audio_path = get_path_from_hash(audio_hash)
        print(audio_path)
        audio_type = tell_file_type(audio_path)
        if background_hashes:
            bg_path = get_path_from_hash(background_hashes[0])
        else:
            bg_path = None
        title = song["Title"]
        artist = song["Artist"]
        album = song["Source"]

        # run ffmpeg
        # convert everything to mp3, as ogg does not support cover images
        if audio_type == ".ogg":
            audio_codec = "libmp3lame"
        else:
            audio_codec = "copy"
        if bg_path:
            subprocess.run([
                "ffmpeg", "-y", "-i", str(audio_path), "-i", str(bg_path),
                "-map", "0:a", "-map", "1:v", "-c:a", audio_codec,
                "-c:v", "copy", "-id3v2_version", "3",
                "-metadata", f"title={title}",
                "-metadata", f"artist={artist}",
                "-metadata", f"album={album}",
                str(args.output / f"{title} - {artist}.mp3"),
            ], check=True, stdout=subprocess.DEVNULL)
        else:
            subprocess.run([
                "ffmpeg", "-y", "-i", str(audio_path),
                "-c:a", audio_codec, "-id3v2_version", "3",
                "-metadata", f"title={title}",
                "-metadata", f"artist={artist}",
                "-metadata", f"album={album}",
                str(args.output / f"{title} - {artist}.mp3"),
            ], check=True, stdout=subprocess.DEVNULL)

if __name__ == "__main__":
    parser = argparse.ArgumentParser(description="Generate music files from song.json")
    parser.add_argument("--json", type=Path, help="Path to json file")
    parser.add_argument("--data", type=Path, help="Path to osu! data directory")
    parser.add_argument("--output", type=Path, help="Output directory for music files")
    args = parser.parse_args()
    args.output.mkdir(parents=True, exist_ok=True)
    main(args)