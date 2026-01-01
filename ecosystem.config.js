// PM2 Ecosystem Config - UBL 3.0 Local Development
// Usage: 
//   source .env && pm2 start ecosystem.config.js
//   pm2 logs --lines 50
//   pm2 monit
//   pm2 stop all

const path = require('path');
const ROOT = __dirname;
const LOG_DIR = path.join(ROOT, '.logs');

module.exports = {
  apps: [
    // ═══════════════════════════════════════════════════════════════
    // UBL KERNEL - The ledger core (must start first)
    // ═══════════════════════════════════════════════════════════════
    {
      name: "ubl-server",
      script: path.join(ROOT, "ubl/kernel/rust/target/release/ubl-server"),
      cwd: ROOT,
      
      // Environment
      env: {
        DATABASE_URL: "postgres:///ubl_ledger?host=/tmp",
        WEBAUTHN_ORIGIN: "http://localhost:3000",
        WEBAUTHN_RP_ID: "localhost",
        RUST_LOG: "info,ubl_server=debug",
        RUST_BACKTRACE: "1"
      },
      
      // Restart policy
      autorestart: true,
      max_restarts: 50,
      min_uptime: "10s",
      restart_delay: 2000,
      exp_backoff_restart_delay: 100,
      
      // Logs with timestamps
      time: true,
      log_date_format: "YYYY-MM-DD HH:mm:ss.SSS",
      out_file: path.join(LOG_DIR, "ubl-server.log"),
      error_file: path.join(LOG_DIR, "ubl-server.error.log"),
      merge_logs: true,
      log_type: "json",
      
      // Process management
      kill_timeout: 5000,
      wait_ready: true,
      listen_timeout: 10000,
      
      // Don't watch - binary doesn't change
      watch: false
    },

    // ═══════════════════════════════════════════════════════════════
    // OFFICE - LLM Operating System (depends on UBL)
    // ═══════════════════════════════════════════════════════════════
    {
      name: "office",
      script: path.join(ROOT, "apps/office/target/release/office"),
      cwd: path.join(ROOT, "apps/office"),
      
      // Environment - uses OFFICE__ prefix with __ separator
      env: {
        OFFICE__SERVER__HOST: "0.0.0.0",
        OFFICE__SERVER__PORT: "8081",
        OFFICE__LLM__PROVIDER: "anthropic",
        OFFICE__LLM__API_KEY: process.env.ANTHROPIC_API_KEY || "",
        OFFICE__UBL__ENDPOINT: "http://localhost:8080",
        DATABASE_URL: "postgres:///ubl_ledger?host=/tmp",
        RUST_LOG: "info,office=debug",
        RUST_BACKTRACE: "1"
      },
      
      // Restart policy - wait for UBL to be ready
      autorestart: true,
      max_restarts: 50,
      min_uptime: "5s",
      restart_delay: 3000,
      exp_backoff_restart_delay: 100,
      
      // Logs
      time: true,
      log_date_format: "YYYY-MM-DD HH:mm:ss.SSS",
      out_file: path.join(LOG_DIR, "office.log"),
      error_file: path.join(LOG_DIR, "office.error.log"),
      merge_logs: true,
      
      // Process management
      kill_timeout: 5000,
      watch: false
    },

    // ═══════════════════════════════════════════════════════════════
    // MESSENGER - React Frontend (Vite dev server)
    // ═══════════════════════════════════════════════════════════════
    {
      name: "messenger",
      script: "npm",
      args: "run dev -- --host",
      cwd: path.join(ROOT, "apps/messenger/frontend"),
      interpreter: "none",
      
      // Environment
      env: {
        VITE_API_URL: "http://localhost:8080",
        PORT: "3000",
        NODE_ENV: "development"
      },
      
      // Restart policy
      autorestart: true,
      max_restarts: 20,
      min_uptime: "5s",
      restart_delay: 2000,
      
      // Logs
      time: true,
      log_date_format: "YYYY-MM-DD HH:mm:ss.SSS",
      out_file: path.join(LOG_DIR, "messenger.log"),
      error_file: path.join(LOG_DIR, "messenger.error.log"),
      merge_logs: true,
      
      // Process management
      kill_timeout: 3000,
      watch: false
    }
  ]
};
