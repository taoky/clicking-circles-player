import { usePlayer } from '@/context/PlayerContext';
import { useCallback } from 'react';
import { FaPlay, FaPause, FaStepForward, FaStepBackward } from 'react-icons/fa';
import { MdLanguage, MdRepeatOn, MdRepeat } from 'react-icons/md';
import { getApiUrl } from '@/config';

export function Player() {
    const {
        currentSong,
        isPlaying,
        progress,
        duration,
        isUnicode,
        isRandom,
        pause,
        resume,
        next,
        previous,
        seek,
        toggleUnicode,
        togglePlayMode,
    } = usePlayer();

    const formatTime = (time: number) => {
        const minutes = Math.floor(time / 60);
        const seconds = Math.floor(time % 60);
        return `${minutes}:${seconds.toString().padStart(2, '0')}`;
    };

    const handleSeek = useCallback(
        (e: React.ChangeEvent<HTMLInputElement>) => {
            const time = parseFloat(e.target.value);
            seek(time);
        },
        [seek]
    );

    if (!currentSong) {
        return null;
    }

    const title = isUnicode ? currentSong.TitleUnicode : currentSong.Title;
    const artist = isUnicode ? currentSong.ArtistUnicode : currentSong.Artist;

    return (
        <div className="bg-gray-900 text-white p-4 border-t border-gray-800 relative">
            {currentSong.BGHashes[0] && (
                <div className="absolute inset-0 opacity-20">
                    <img
                        src={getApiUrl(`image/${currentSong.BGHashes[0]}`)}
                        alt="Background"
                        className="w-full h-full object-cover"
                    />
                </div>
            )}
            <div className="max-w-3xl mx-auto relative z-10">
                <div className="flex justify-between items-center mb-2">
                    <div className="flex-1">
                        <h3 className="text-lg font-bold truncate">{title}</h3>
                        <p className="text-sm text-gray-300 truncate">{artist}</p>
                    </div>
                </div>
                <div className="flex items-center gap-2">
                    <span className="text-sm">{formatTime(progress)}</span>
                    <input
                        type="range"
                        min="0"
                        max={duration}
                        value={progress}
                        onChange={handleSeek}
                        className="flex-1 h-2 bg-gray-700 rounded-lg appearance-none cursor-pointer"
                    />
                    <span className="text-sm">{formatTime(duration)}</span>
                </div>
                <div className="flex justify-center items-center gap-4 mt-2">
                    <button
                        onClick={toggleUnicode}
                        className="p-2 hover:bg-gray-700 rounded-full"
                        title="Toggle Unicode"
                    >
                        <MdLanguage size={20} />
                    </button>
                    <button
                        onClick={previous}
                        className="p-2 hover:bg-gray-700 rounded-full"
                    >
                        <FaStepBackward size={20} />
                    </button>
                    <button
                        onClick={isPlaying ? pause : resume}
                        className="p-3 hover:bg-gray-700 rounded-full"
                    >
                        {isPlaying ? <FaPause size={24} /> : <FaPlay size={24} />}
                    </button>
                    <button
                        onClick={next}
                        className="p-2 hover:bg-gray-700 rounded-full"
                    >
                        <FaStepForward size={20} />
                    </button>
                    <button
                        onClick={togglePlayMode}
                        className={`p-2 hover:bg-gray-700 rounded-full`}
                        title={isRandom ? "Random play" : "Repeat play"}
                    >
                        {/* Repeat off and Repeat on icons */}
                        {isRandom ? <MdRepeat size={20} /> : <MdRepeatOn size={20} />}
                    </button>
                </div>
            </div>
        </div>
    );
} 