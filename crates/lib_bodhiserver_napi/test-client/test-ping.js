#!/usr/bin/env node

/**
 * Simple ping test for lib_bodhiserver_napi
 * Tests the /ping endpoint without authentication
 */

import { BodhiApp } from 'lib_bodhiserver_napi';
import http from 'http';

async function testPingEndpoint() {
  console.log('🏓 Testing /ping endpoint...');
  
  let app;
  try {
    // Create basic configuration
    const config = {
      env_type: 'development',
      app_type: 'container',
      app_version: '1.0.0-test',
      auth_url: 'https://dev-id.getbodhi.app',
      auth_realm: 'bodhi',
      bodhi_home: '/tmp/bodhi_ping_test',
      encryption_key: 'test-encryption-key-ping',
      port: 0, // Use random port
      oauth_client_id: 'test_client_id',
      oauth_client_secret: 'test_client_secret',
      app_status: 'Ready'
    };

    // Initialize app
    app = new BodhiApp();
    await app.initialize(config);
    console.log('✅ App initialized');
    
    // Start server
    const serverUrl = await app.startServer('127.0.0.1', 0);
    console.log(`🌐 Server started at: ${serverUrl}`);
    
    // Extract port from URL
    const url = new URL(serverUrl);
    const port = parseInt(url.port);
    
    // Test ping endpoint
    const pingResponse = await makeHttpRequest('127.0.0.1', port, '/ping');
    console.log('📡 Ping response:', pingResponse);
    
    if (pingResponse.includes('pong')) {
      console.log('✅ Ping test passed!');
    } else {
      throw new Error('Ping response does not contain "pong"');
    }
    
  } catch (error) {
    console.error('❌ Ping test failed:', error);
    throw error;
  } finally {
    if (app) {
      try {
        await app.shutdown();
        console.log('✅ Server shutdown');
      } catch (shutdownError) {
        console.error('⚠️ Shutdown error:', shutdownError);
      }
    }
  }
}

function makeHttpRequest(host, port, path) {
  return new Promise((resolve, reject) => {
    const options = {
      hostname: host,
      port: port,
      path: path,
      method: 'GET',
      timeout: 5000
    };

    const req = http.request(options, (res) => {
      let data = '';
      res.on('data', (chunk) => {
        data += chunk;
      });
      res.on('end', () => {
        resolve(data);
      });
    });

    req.on('error', (error) => {
      reject(error);
    });

    req.on('timeout', () => {
      req.destroy();
      reject(new Error('Request timeout'));
    });

    req.end();
  });
}

async function main() {
  console.log('🚀 Starting Ping Test');
  console.log('=' .repeat(50));
  
  try {
    await testPingEndpoint();
    console.log('🎉 Ping test completed successfully!');
  } catch (error) {
    console.error('💥 Ping test failed:', error);
    process.exit(1);
  }
}

main();
