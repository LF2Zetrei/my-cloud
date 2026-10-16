import type { NextConfig } from "next";

const nextConfig: NextConfig = {
    async rewrites() {
        return [
            { source: "/api/:path*",     destination: "http://localhost:8080/api/:path*" },
            { source: "/metrics/:path*", destination: "http://localhost:8080/metrics/:path*" },
            { source: "/metrics",        destination: "http://localhost:8080/metrics" },
            { source: "/storage/:path*", destination: "http://localhost:8080/storage/:path*" },
            { source: "/health",         destination: "http://localhost:8080/health" },
        ]
    },
};

export default nextConfig;
