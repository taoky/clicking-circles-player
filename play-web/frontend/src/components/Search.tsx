import React from 'react';
import { usePlayer } from '@/context/PlayerContext';
import { FaSearch } from 'react-icons/fa';

export function Search() {
    const { searchQuery, setSearchQuery } = usePlayer();

    return (
        <div className="p-4 bg-gray-800 border-b border-gray-700">
            <div className="relative max-w-3xl mx-auto">
                <input
                    type="text"
                    placeholder="Search songs..."
                    value={searchQuery}
                    onChange={(e) => setSearchQuery(e.target.value)}
                    className="w-full px-4 py-2 pl-10 bg-gray-700 text-white rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
                />
                <FaSearch className="absolute left-3 top-3 text-gray-400" />
            </div>
        </div>
    );
} 