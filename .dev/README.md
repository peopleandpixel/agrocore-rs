# .dev directory — persistent development infrastructure
# This file prevents accidental deletion of pid files
# 
# Contents:
#   docker-compose.yml  — PostgreSQL + NATS (persistent volume, not deleted on restart)
#   dashboard.sh        — Interactive TUI dashboard (no tmux!)
#   services.sh         — Service start/stop/restart with PID tracking
#   pids/               — PID files for each service (created at runtime)
