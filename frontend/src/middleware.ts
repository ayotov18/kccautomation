import { NextRequest, NextResponse } from 'next/server';

export const config = {
  matcher: '/api/:path*',
};

export function middleware(req: NextRequest) {
  const apiOrigin = process.env.INTERNAL_API_URL || 'http://localhost:3000';
  const url = new URL(req.nextUrl.pathname + req.nextUrl.search, apiOrigin);
  return NextResponse.rewrite(url);
}
