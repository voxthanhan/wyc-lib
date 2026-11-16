'use strict';
const http = require('node:http');
const api = require('wyc-lib');
const server = http.createServer(async (req, res) => {
  res.setHeader('Content-Type', 'application/json');
  res.setHeader('Cache-Control', 'no-store');
  if (req.method !== 'POST' || req.url !== '/analyze') {
    res.writeHead(404).end('{}'); return;
  }
  try {
    let body = '';
    for await (const chunk of req) {
      body += chunk;
      if (body.length > 1024 * 1024) { res.writeHead(413).end('{}'); return; }
    }
    const result = api.analyze_batch(JSON.parse(body));
    res.writeHead(200).end(JSON.stringify(result));
  } catch (error) {
    const {code = 'invalid_json', path = 'request'} = error;
    res.writeHead(400).end(JSON.stringify({error: {code, path}}));
  }
});
server.listen(Number(process.env.PORT || 3000), '127.0.0.1');
server.listen(Number(process.env.PORT || 0), '127.0.0.1', () => {
  console.log(JSON.stringify({port: server.address().port}));
});
process.on('SIGTERM', () => server.close());
