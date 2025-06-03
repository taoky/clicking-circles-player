import { createContext, useContext, useState, useCallback, ReactNode, useRef, useEffect } from 'react';
import { Song } from '@/types/song';
import { AudioService } from '@/services/AudioService';
import { getDataUrl, getHashPath } from '@/config';

interface PlayerContextType {
    currentSong: Song | null;
    isPlaying: boolean;
    progress: number;
    duration: number;
    songs: Song[];
    currentIndex: number;
    isUnicode: boolean;
    isRandom: boolean;
    searchQuery: string;
    setSearchQuery: (query: string) => void;
    play: (index: number) => void;
    pause: () => void;
    resume: () => void;
    next: () => void;
    previous: () => void;
    seek: (time: number) => void;
    toggleUnicode: () => void;
    togglePlayMode: () => void;
    setSongs: (songs: Song[]) => void;
}

const PlayerContext = createContext<PlayerContextType | undefined>(undefined);

// Fisher-Yates shuffle algorithm
function shuffleArray<T>(array: T[]): T[] {
    const newArray = [...array];
    for (let i = newArray.length - 1; i > 0; i--) {
        const j = Math.floor(Math.random() * (i + 1));
        [newArray[i], newArray[j]] = [newArray[j], newArray[i]];
    }
    return newArray;
}

export function PlayerProvider({ children }: { children: ReactNode }) {
    const [currentSong, setCurrentSong] = useState<Song | null>(null);
    const [isPlaying, setIsPlaying] = useState(false);
    const [progress, setProgress] = useState(0);
    const [duration, setDuration] = useState(0);
    const [songs, setSongs] = useState<Song[]>([]);
    const [currentIndex, setCurrentIndex] = useState(-1);
    const [isUnicode, setIsUnicode] = useState(false);
    const [isRandom, setIsRandom] = useState(true);
    const [searchQuery, setSearchQuery] = useState('');
    const audioRef = useRef<AudioService | null>(null);
    const animationFrameRef = useRef<number>(0);
    const originalSongs = useRef<Song[]>([]);
    const playRef = useRef<((index: number) => Promise<void>) | undefined>(undefined);


    // Initialize AudioService
    useEffect(() => {
        audioRef.current = new AudioService();

        // Set up event handlers
        audioRef.current.onDurationChange((newDuration) => {
            setDuration(newDuration);
        });

        audioRef.current.onTimeUpdate((newTime) => {
            setProgress(newTime);
        });

        return () => {
            audioRef.current?.destroy();
        };
    }, []);

    // Function to set songs with initial shuffle
    const handleSetSongs = useCallback((newSongs: Song[]) => {
        originalSongs.current = [...newSongs];
        const shuffledSongs = shuffleArray(newSongs);
        setSongs(shuffledSongs);
    }, []);

    // Handle play mode toggle
    const togglePlayMode = useCallback(() => {
        setIsRandom(prev => !prev);
    }, []);

    useEffect(() => {
        const step = () => {
            if (audioRef.current && audioRef.current.isPlaying()) {
                setProgress(audioRef.current.getCurrentTime());
                animationFrameRef.current = requestAnimationFrame(step);
            }
        };

        if (isPlaying) {
            animationFrameRef.current = requestAnimationFrame(step);
        }

        return () => {
            if (animationFrameRef.current) {
                cancelAnimationFrame(animationFrameRef.current);
            }
        };
    }, [isPlaying]);

    // Helper function to get next index
    const getNextIndex = useCallback((currentIdx: number) => {
        if (isRandom) {
            return (currentIdx + 1) % songs.length;
        }
        return currentIdx;
    }, [isRandom, songs.length]);

    const next = useCallback(() => {
        setCurrentIndex(prevIndex => {
            const nextIndex = getNextIndex(prevIndex);
            playRef.current?.(nextIndex);
            return nextIndex;
        });
    }, [getNextIndex]);

    const play = useCallback(async (index: number) => {
        if (!audioRef.current) return;

        try {
            // Reset states before loading new song
            setProgress(0);
            setDuration(0);

            const song = songs[index];

            audioRef.current.stop();
            await audioRef.current.load(getDataUrl(`files/${getHashPath(song.AudioHash)}`));
            await audioRef.current.play();
            setCurrentSong(song);
            setCurrentIndex(index);
            setIsPlaying(true);

            const thisIndex = index;
            audioRef.current.onEnd(() => {
                setIsPlaying(false);
                const nextIndex = getNextIndex(thisIndex);
                playRef.current?.(nextIndex);
            });
        } catch (error) {
            console.error('Error playing audio:', error);
            next();
        }
    }, [songs, getNextIndex, next]);

    // Keep playRef in sync with play function
    useEffect(() => {
        playRef.current = play;
    }, [play]);

    const pause = useCallback(() => {
        audioRef.current?.pause();
        setIsPlaying(false);
    }, []);

    const resume = useCallback(() => {
        audioRef.current?.play();
        setIsPlaying(true);
    }, []);

    const seek = useCallback((time: number) => {
        if (audioRef.current) {
            audioRef.current.seek(time);
            setProgress(time);
        }
    }, []);

    const previous = useCallback(() => {
        if (currentIndex > 0) {
            play(currentIndex - 1);
        } else if (songs.length > 0) {
            play(songs.length - 1);
        }
    }, [currentIndex, songs, play]);

    const toggleUnicode = useCallback(() => {
        setIsUnicode(prev => !prev);
    }, []);

    // Update media session metadata when current song changes
    useEffect(() => {
        if ('mediaSession' in navigator && currentSong) {
            const title = isUnicode ? currentSong.TitleUnicode : currentSong.Title;
            const artist = isUnicode ? currentSong.ArtistUnicode : currentSong.Artist;

            navigator.mediaSession.metadata = new MediaMetadata({
                title,
                artist,
                album: currentSong.Source,
                // TODO: trim image to square
                // artwork: currentSong.BGHashes[0] ? [
                //     {
                //         src: getDataUrl(`files/${getHashPath(currentSong.BGHashes[0])}`),
                //         type: 'image/jpeg',
                //     }
                // ] : undefined
            });
        }
    }, [currentSong, isUnicode]);

    // Set up media session action handlers
    useEffect(() => {
        if ('mediaSession' in navigator) {
            navigator.mediaSession.setActionHandler('play', resume);
            navigator.mediaSession.setActionHandler('pause', pause);
            navigator.mediaSession.setActionHandler('previoustrack', previous);
            navigator.mediaSession.setActionHandler('nexttrack', next);
            navigator.mediaSession.setActionHandler('seekto', (details) => {
                if (details.seekTime) {
                    seek(details.seekTime);
                }
            });

            return () => {
                // Cleanup handlers when component unmounts
                navigator.mediaSession.setActionHandler('play', null);
                navigator.mediaSession.setActionHandler('pause', null);
                navigator.mediaSession.setActionHandler('previoustrack', null);
                navigator.mediaSession.setActionHandler('nexttrack', null);
                navigator.mediaSession.setActionHandler('seekto', null);
            };
        }
    }, [pause, resume, previous, next, seek]);

    // Update media session playback state
    useEffect(() => {
        if ('mediaSession' in navigator) {
            navigator.mediaSession.playbackState = isPlaying ? 'playing' : 'paused';
        }
    }, [isPlaying]);

    // Update media session position state
    useEffect(() => {
        if ('mediaSession' in navigator && duration > 0 && duration !== Infinity) {
            const validPosition = isFinite(progress) ? progress : 0;
            const validDuration = isFinite(duration) ? duration : 0;

            try {
                navigator.mediaSession.setPositionState({
                    duration: validDuration,
                    position: validPosition,
                    playbackRate: 1,
                });
            } catch (error) {
                console.warn('Failed to update media session position state:', error);
            }
        }
    }, [progress, duration]);

    return (
        <PlayerContext.Provider
            value={{
                currentSong,
                isPlaying,
                progress,
                duration,
                songs,
                currentIndex,
                isUnicode,
                isRandom,
                searchQuery,
                setSearchQuery,
                play,
                pause,
                resume,
                next,
                previous,
                seek,
                toggleUnicode,
                togglePlayMode,
                setSongs: handleSetSongs,
            }}
        >
            {children}
        </PlayerContext.Provider>
    );
}

export function usePlayer() {
    const context = useContext(PlayerContext);
    if (context === undefined) {
        throw new Error('usePlayer must be used within a PlayerProvider');
    }
    return context;
} 