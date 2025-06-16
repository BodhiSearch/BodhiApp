import { BodhiApp, AppConfig } from '@bodhiapp/server-bindings';
import fetch from 'node-fetch';

/**
 * Test the NAPI-RS FFI bindings for BodhiApp server functionality.
 * 
 * This test demonstrates:
 * 1. Creating and initializing a BodhiApp instance
 * 2. Starting the HTTP server
 * 3. Making a request to the /ping endpoint
 * 4. Shutting down the server cleanly
 */
async function testBodhiAppFFI(): Promise<void> {
  console.log('🚀 Starting BodhiApp NAPI-RS FFI Test');
  
  let app: BodhiApp | null = null;
  
  try {
    // Step 1: Create BodhiApp instance
    console.log('📦 Creating BodhiApp instance...');
    app = new BodhiApp();
    console.log(`✅ App created with status: ${app.getStatus()}`);
    
    // Step 2: Initialize with development configuration
    console.log('⚙️  Initializing BodhiApp...');

    // Get the absolute path to the llama-server executable
    const path = require('path');
    const execLookupPath = path.resolve(__dirname, '../../../crates/bodhi/src-tauri/bin');
    console.log(`🔧 Using exec lookup path: ${execLookupPath}`);

    const testPort = 54321; // Use a fixed port to avoid port extraction complexity
    const config: AppConfig = {
      envType: 'development',
      appType: 'container',
      appVersion: '1.0.0-napi-test',
      authUrl: 'https://dev-id.getbodhi.app',
      authRealm: 'bodhi',
      encryptionKey: 'test-encryption-key',
      execLookupPath: execLookupPath,
      port: testPort
    };
    
    await app.initialize(config);
    console.log(`✅ App initialized with status: ${app.getStatus()}`);
    
    // Step 3: Start the server with embedded assets
    console.log('🌐 Starting HTTP server with embedded UI assets...');
    const serverUrl = await app.startServer('127.0.0.1', testPort);
    console.log(`✅ Server started at: ${serverUrl}`);
    console.log(`📊 App status: ${app.getStatus()}`);
    
    // Step 4: Test the /ping endpoint
    console.log('🏓 Testing /ping endpoint...');
    const pingUrl = `${serverUrl}/ping`;
    console.log(`📡 Making request to: ${pingUrl}`);
    
    const response = await fetch(pingUrl);
    const responseText = await response.text();
    
    console.log(`📥 Response status: ${response.status}`);
    console.log(`📄 Response body: ${responseText}`);
    
    if (response.status === 200) {
      console.log('✅ /ping endpoint test successful!');
    } else {
      console.error('❌ /ping endpoint test failed!');
      process.exit(1);
    }
    
    // Step 5: Shutdown the server
    console.log('🛑 Shutting down server...');
    await app.shutdown();
    console.log(`✅ Server shutdown complete. Final status: ${app.getStatus()}`);
    
    console.log('🎉 BodhiApp NAPI-RS FFI Test completed successfully!');
    
  } catch (error) {
    console.error('❌ Test failed with error:', error);
    
    // Attempt cleanup
    if (app) {
      try {
        await app.shutdown();
        console.log('🧹 Cleanup completed');
      } catch (cleanupError) {
        console.error('⚠️  Cleanup failed:', cleanupError);
      }
    }
    
    process.exit(1);
  }
}

// Handle process signals for graceful shutdown
process.on('SIGINT', () => {
  console.log('\n🛑 Received SIGINT, exiting...');
  process.exit(0);
});

process.on('SIGTERM', () => {
  console.log('\n🛑 Received SIGTERM, exiting...');
  process.exit(0);
});

// Run the test
if (require.main === module) {
  testBodhiAppFFI().catch((error) => {
    console.error('💥 Unhandled error:', error);
    process.exit(1);
  });
}
