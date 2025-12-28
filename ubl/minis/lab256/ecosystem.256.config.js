module.exports = {
  apps: [{
    name: "ubl-server",
    script: "/opt/ubl/bin/ubl-server",
    env: {
      ENGINE_CONFIG: "/opt/ubl/etc/engine.yaml"
    },
    watch: false,
    autorestart: true,
    max_restarts: 10,
    out_file: "/opt/ubl/logs/ubl-server.out.log",
    error_file: "/opt/ubl/logs/ubl-server.err.log"
  }]
}