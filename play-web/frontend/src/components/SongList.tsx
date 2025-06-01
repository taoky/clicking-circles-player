import { useCallback } from 'react';
import { usePlayer } from '@/context/PlayerContext';
import { Song } from '@/types/song';
import { getApiUrl } from '@/config';

export function SongList() {
    const { songs, currentIndex, play, isUnicode, searchQuery } = usePlayer();

    const filteredSongs = songs.map((song, index) => [song, index] as [Song, number]).filter(([song]) => {
        const query = searchQuery.toLowerCase();
        return (
            song.Title.toLowerCase().includes(query) ||
            song.Artist.toLowerCase().includes(query) ||
            song.Source.toLowerCase().includes(query) ||
            song.TitleUnicode.toLowerCase().includes(query) ||
            song.ArtistUnicode.toLowerCase().includes(query) ||
            song.Tags.some(tag => tag.toLowerCase().includes(query))
        );
    });

    const handlePlay = useCallback((index: number) => {
        play(index);
    }, [play]);

    const getSongTitle = (song: Song) => {
        return isUnicode ? song.TitleUnicode : song.Title;
    };

    const getSongArtist = (song: Song) => {
        return isUnicode ? song.ArtistUnicode : song.Artist;
    };

    return (
        <div className="h-full overflow-y-auto overscroll-contain">
            <div className="divide-y divide-gray-700">
                {filteredSongs.map(([song, index]) => (
                    <button
                        key={`${song.AudioHash}-${index}`}
                        onClick={() => handlePlay(index)}
                        className={`w-full text-left p-4 hover:bg-gray-700 transition-colors ${
                            index === currentIndex ? 'bg-gray-700' : ''
                        }`}
                    >
                        <div className="flex items-center">
                            {song.BGHashes[0] && (
                                <img
                                    alt={song.Title}
                                    src={getApiUrl(`image/${song.BGHashes[0]}`)}
                                    className="w-12 h-12 object-cover rounded mr-4"
                                    loading="lazy"
                                />
                            )}
                            <div className="flex-1 min-w-0">
                                <h3 className="text-white font-medium truncate">
                                    {getSongTitle(song)}
                                </h3>
                                <p className="text-gray-400 text-sm truncate">
                                    {getSongArtist(song)}
                                </p>
                                {song.Source && (
                                    <p className="text-gray-500 text-xs truncate">
                                        {song.Source}
                                    </p>
                                )}
                            </div>
                        </div>
                    </button>
                ))}
            </div>
        </div>
    );
} 