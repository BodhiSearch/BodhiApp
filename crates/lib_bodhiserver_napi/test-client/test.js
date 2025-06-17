#!/usr/bin/env node

/**
 * Test client for lib_bodhiserver_napi enhanced configuration
 * This tests the JavaScript/TypeScript interface for the enhanced builder methods
 */

import { BodhiApp } from 'lib_bodhiserver_napi';

async function testEnhancedConfiguration() {
  console.log('🧪 Testing Enhanced Configuration...');
  
  try {
    // Create enhanced configuration
    const config = {
      env_type: 'development',
      app_type: 'container',
      app_version: '1.0.0-test',
      auth_url: 'https://dev-id.getbodhi.app',
      auth_realm: 'bodhi',
      bodhi_home: '/tmp/bodhi_js_test',
      // Enhanced configuration fields
      environment_vars: {
        'TEST_VAR': 'test_value_js',
        'NODE_ENV': 'test',
        'BODHI_ENCRYPTION_KEY': 'test-encryption-key-js',
        'BODHI_EXEC_LOOKUP_PATH': '/tmp',
        'BODHI_PORT': '54323',
      },
      app_settings: {
        'BODHI_PORT': '54323',
        'BODHI_LOG_LEVEL': 'debug'
      },
      system_settings: {
        'BODHI_ENV_TYPE': 'development'
      },
      oauth_client_id: 'test_client_id_js',
      oauth_client_secret: 'test_client_secret_js',
      app_status: 'Ready'
    };

    console.log('📋 Configuration:', JSON.stringify(config, null, 2));

    // Test app initialization
    const app = new BodhiApp();
    console.log('✅ BodhiApp instance created');
    
    // Test initialization with enhanced config
    await app.initialize(config);
    console.log('✅ App initialized with enhanced configuration');
    
    // Test status
    const status = app.getStatus();
    console.log(`📊 App status: ${status} (1 = Ready)`);
    
    if (status !== 1) {
      throw new Error(`Expected status 1 (Ready), got ${status}`);
    }
    
    // Test shutdown
    await app.shutdown();
    console.log('✅ App shutdown successfully');
    
    const finalStatus = app.getStatus();
    console.log(`📊 Final status: ${finalStatus} (3 = Shutdown)`);
    
    if (finalStatus !== 3) {
      throw new Error(`Expected final status 3 (Shutdown), got ${finalStatus}`);
    }
    
    console.log('🎉 All enhanced configuration tests passed!');
    
  } catch (error) {
    console.error('❌ Test failed:', error);
    process.exit(1);
  }
}

async function testBasicConfiguration() {
  console.log('🧪 Testing Basic Configuration...');
  
  try {
    // Test with minimal configuration
    const config = {
      env_type: 'development',
      app_type: 'container',
      app_version: '1.0.0-test',
      auth_url: 'https://dev-id.getbodhi.app',
      auth_realm: 'bodhi',
      bodhi_home: '/tmp/bodhi_js_basic',
      environment_vars: {
        'BODHI_ENCRYPTION_KEY': 'test-encryption-key-basic',
      }
    };

    const app = new BodhiApp();
    await app.initialize(config);
    console.log('✅ Basic configuration test passed');
    
    await app.shutdown();
    console.log('✅ Basic shutdown test passed');
    
  } catch (error) {
    console.error('❌ Basic test failed:', error);
    process.exit(1);
  }
}

async function main() {
  console.log('🚀 Starting lib_bodhiserver_napi JavaScript/TypeScript Interface Tests');
  console.log('=' .repeat(80));
  
  await testBasicConfiguration();
  console.log();
  await testEnhancedConfiguration();
  
  console.log();
  console.log('🎉 All tests completed successfully!');
}

main().catch(error => {
  console.error('💥 Test suite failed:', error);
  process.exit(1);
});
