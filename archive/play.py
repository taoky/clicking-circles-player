import json
import sys
import random
from pathlib import Path
# from shutil import copyfile
import subprocess

# Get osu! files folder, like ~/.var/app/sh.ppy.osu/data/osu/files/
osu_folder = Path(sys.argv[1])

def get_path_from_hash(hash):
    return osu_folder / hash[0] / hash[:2] / hash

with open("./song.json") as f:
    songs = json.load(f)

# shuffle songs
random.shuffle(songs)

for song in songs:
    audio_hash = song["AudioHash"]
    background_hash = random.choice(song["BGHashes"])
    title = song["Metadata"]["Title"]
    title_unicode = song["Metadata"]["TitleUnicode"]
    artist = song["Metadata"]["Artist"]
    artist_unicode = song["Metadata"]["ArtistUnicode"]
    source = song["Metadata"]["Source"]
    tags = song["Metadata"]["Tags"]

    audio = get_path_from_hash(audio_hash)
    background = get_path_from_hash(background_hash)

    # show image
    subprocess.run(["viu", "--width", "80", background])
    # show metadata
    print(f"Title: {title} ({title_unicode})")
    print(f"Artist: {artist} ({artist_unicode})")
    print(f"Source: {source}")
    # print(f"Tags: {tags}")

    # Play with mpv
    subprocess.run(["mpv", "--no-video", audio])
