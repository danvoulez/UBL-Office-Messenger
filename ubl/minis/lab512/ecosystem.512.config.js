module.exports = {
  apps: [{
    name: "ubl-runner",
    script: "/opt/ubl/bin/ubl-runner",
    env: {
      RUNNER_CONFIG: "/opt/ubl/etc/runner.yaml"
    },
    watch: false,
    autorestart: true,
    max_restarts: 10,
    out_file: "/opt/ubl/logs/ubl-runner.out.log",
    error_file: "/opt/ubl/logs/ubl-runner.err.log"
  }]
}