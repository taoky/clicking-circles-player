'use client';

import { useEffect } from 'react';
import axios from 'axios';
import { PlayerProvider } from '@/context/PlayerContext';
import { Player } from '@/components/Player';
import { SongList } from '@/components/SongList';
import { Search } from '@/components/Search';
import { usePlayer } from '@/context/PlayerContext';
import { getDataUrl } from '@/config';

function MainContent() {
    const { setSongs } = usePlayer();

    useEffect(() => {
        const fetchSongs = async () => {
            try {
                const response = await axios.get(getDataUrl('song.json'));
                setSongs(response.data);
            } catch (error) {
                console.error('Error fetching songs:', error);
            }
        };

        fetchSongs();
    }, [setSongs]);

    return (
        <main className="flex flex-col h-screen bg-gray-900 text-white">
            {/* Search section - fixed at top */}
            <div className="flex-none">
                <Search />
            </div>
            
            {/* Song list section - scrollable */}
            <div className="flex-1 overflow-y-auto">
                <SongList />
            </div>
            
            {/* Player section - fixed at bottom */}
            <div className="flex-none">
                <Player />
            </div>
        </main>
    );
}

export default function Home() {
    return (
        <PlayerProvider>
            <MainContent />
        </PlayerProvider>
    );
}
