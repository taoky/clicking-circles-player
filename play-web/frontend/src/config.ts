// Use environment variable for API URL, fallback to relative path
const apiBase = process.env.NEXT_PUBLIC_API_URL || '/api';

// Remove /api suffix if it's already in the environment variable
const normalizedApiBase = apiBase.endsWith('/api') ? apiBase.slice(0, -4) : apiBase;

export const getApiUrl = (path: string) => `${normalizedApiBase}/api/${path}`;
