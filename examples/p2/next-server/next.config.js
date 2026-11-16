/** @type {import('next').NextConfig} */
module.exports = {
  serverExternalPackages: ['wyc-lib'],
  // Bound production build workers on small CI runners.
  experimental: {cpus: 2},
};
