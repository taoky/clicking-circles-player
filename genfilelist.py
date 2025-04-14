#!/usr/bin/env python3

# Usage:
# ./genfilelist.py ./song.json > filelist
# tar cvaf song.tar.zst -C ~/.var/app/sh.ppy.osu/data/osu/files/ --files-from=filelist

import argparse
from pathlib import Path
import json

def main(args):
    with open(args.json) as f:
        songs = json.load(f)
    
    def get_path_from_hash(hash):
        return Path(hash[0]) / hash[:2] / hash
    
    for song in songs:
        audio_hash = song["AudioHash"]
        background_hashes = song["BGHashes"]
        print(get_path_from_hash(audio_hash))
        for background_hash in background_hashes:
            print(get_path_from_hash(background_hash))

if __name__ == "__main__":
    parser = argparse.ArgumentParser(description="Generate a list for tar/rsync/...")
    parser.add_argument("json", type=Path, help="Path to json file")
    args = parser.parse_args()
    main(args)
