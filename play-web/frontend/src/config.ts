// Use environment variable for API URL, fallback to relative path
const apiBase = process.env.NEXT_PUBLIC_API_URL || '/api';

// Remove /api suffix if it's already in the environment variable
const normalizedApiBase = apiBase.endsWith('/api') ? apiBase.slice(0, -4) : apiBase;

export const getApiUrl = (path: string) => `${normalizedApiBase}/api/${path}`;

// Use environment variable for data URL, fallback to relative path
const dataBase = process.env.NEXT_PUBLIC_DATA_URL || '/data';

// Remove trailing slash if present
const normalizedDataBase = dataBase.endsWith('/') ? dataBase.slice(0, -1) : dataBase;

export const getDataUrl = (path: string) => `${normalizedDataBase}/${path}`;

// Helper function to generate file path in the format hash[0]/hash[:2]/hash
export const getHashPath = (hash: string) => {
    return `${hash[0]}/${hash.slice(0, 2)}/${hash}`;
};
