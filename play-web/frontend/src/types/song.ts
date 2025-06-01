export interface Metadata {
    Title: string;
    TitleUnicode: string;
    Artist: string;
    ArtistUnicode: string;
    Source: string;
    Tags: string[];
}

export interface Song {
    Title: string;
    TitleUnicode: string;
    Artist: string;
    ArtistUnicode: string;
    Source: string;
    Tags: string[];
    AudioHash: string;
    BGHashes: string[];
}

export interface SongWithIndex {
    index: number;
    song: Song;
} 