/**
 * Runtime WS auth token — read from server env at request time.
 * Never baked into the client bundle (no NEXT_PUBLIC_WS_TOKEN in production).
 */
export async function GET() {
  const token = process.env.ARGUS_WS_TOKEN;
  if (!token) {
    return Response.json({ error: 'WS token not configured' }, { status: 503 });
  }
  return Response.json(
    { token },
    {
      headers: {
        'Cache-Control': 'no-store, no-cache, must-revalidate',
      },
    },
  );
}