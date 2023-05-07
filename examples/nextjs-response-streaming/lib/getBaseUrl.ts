import { cache } from 'react';

export const getBaseUrl = cache(() =>
    process.env.VERCEL_URL
        ? `https://app-dir.vercel.app`
        : `http://127.0.0.1:${process.env.PORT ?? 3000}`,
);
