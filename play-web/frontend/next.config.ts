const withPWA = require("@ducanh2912/next-pwa").default({
  dest: "public",
  workboxOptions: {
    // cache /api/image and /api/audio/
    runtimeCaching: [
      {
        urlPattern: /\/api\/image/,
        handler: 'CacheFirst',
      },
      {
        urlPattern: /\/api\/audio/,
        handler: 'CacheFirst',
      },
    ]
  }
});

const config = withPWA({
  reactStrictMode: true,
});

export default config;
