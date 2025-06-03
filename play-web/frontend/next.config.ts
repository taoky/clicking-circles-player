const withPWA = require("@ducanh2912/next-pwa").default({
  dest: "public",
  workboxOptions: {
    runtimeCaching: [
      {
        urlPattern: /\/data\/files/,
        handler: 'CacheFirst',
      }
    ]
  }
});

const config = withPWA({
  reactStrictMode: true,
  output: 'export',
  // Disable image optimization since we're serving static files
  images: {
    unoptimized: true,
  },
  // Enable source maps in production
  productionBrowserSourceMaps: true,
});

export default config;
