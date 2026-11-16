import api from 'wyc-lib';

export const runtime = 'nodejs';
export const dynamic = 'force-dynamic';

export async function POST(request) {
  try {
    return Response.json(api.analyze_batch(await request.json()), {
      headers: {'Cache-Control': 'no-store'},
    });
  } catch (error) {
    const {code = 'invalid_json', path = 'request'} = error;
    return Response.json({error: {code, path}}, {status: 400});
  }
}
