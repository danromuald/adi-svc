const { createProxyMiddleware } = require('http-proxy-middleware');

module.exports = function(app) {
  // Proxy API requests
  app.use(
    '/api',
    createProxyMiddleware({
      target: 'http://localhost:8080',
      changeOrigin: true,
      logLevel: 'silent',
      onError: (err, req, res) => {
        console.error('API Proxy error:', err.message);
        res.status(503).json({
          error: 'Backend not available',
          message: 'Please ensure the backend is running on port 8080.',
          details: err.message
        });
      },
    })
  );
  
  // Proxy health check
  app.use(
    '/health',
    createProxyMiddleware({
      target: 'http://localhost:8080',
      changeOrigin: true,
      logLevel: 'silent',
      onError: (err, req, res) => {
        res.status(503).json({
          error: 'Backend not available',
          status: 'unhealthy'
        });
      },
    })
  );
};

