#!/bin/bash
# Manual fault injection script for interactive testing

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

echo "🔥 Fault Injection Tool"
echo ""

PS3='Select fault to inject: '
options=(
    "Network Latency (500ms)"
    "Network Latency (2s)"
    "Network Partition (UBL ↔ Office)"
    "Kill UBL Kernel (SIGKILL)"
    "Kill Office (SIGKILL)"
    "Stop PostgreSQL"
    "CPU Pressure (UBL)"
    "Memory Pressure (Office)"
    "Remove All Faults"
    "Quit"
)

select opt in "${options[@]}"
do
    case $opt in
        "Network Latency (500ms)")
            echo -e "${YELLOW}Injecting 500ms latency...${NC}"
            # Using toxiproxy
            curl -X POST http://localhost:8474/proxies/ubl/toxics \
                -d '{"name":"latency_500","type":"latency","attributes":{"latency":500}}'
            echo -e "${GREEN}✅ Latency injected${NC}"
            ;;
        "Network Latency (2s)")
            echo -e "${YELLOW}Injecting 2s latency...${NC}"
            curl -X POST http://localhost:8474/proxies/ubl/toxics \
                -d '{"name":"latency_2000","type":"latency","attributes": {"latency":2000}}'
            echo -e "${GREEN}✅ Latency injected${NC}"
            ;;
        "Network Partition (UBL ↔ Office)")
            echo -e "${YELLOW}Creating network partition...${NC}"
            docker network disconnect chaos-net ubl-kernel 2>/dev/null || true
            echo -e "${GREEN}✅ Partition created${NC}"
            echo "To heal: docker network connect chaos-net ubl-kernel"
            ;;
        "Kill UBL Kernel (SIGKILL)")
            echo -e "${RED}Killing UBL Kernel...${NC}"
            docker kill -s SIGKILL ubl-kernel
            echo -e "${GREEN}✅ UBL Kernel killed${NC}"
            echo "To restart: docker-compose -f docker-compose.chaos.yml up -d ubl-kernel"
            ;;
        "Kill Office (SIGKILL)")
            echo -e "${RED}Killing Office...${NC}"
            docker kill -s SIGKILL office
            echo -e "${GREEN}✅ Office killed${NC}"
            echo "To restart: docker-compose -f docker-compose.chaos.yml up -d office"
            ;;
        "Stop PostgreSQL")
            echo -e "${RED}Stopping PostgreSQL...${NC}"
            docker-compose -f docker-compose.chaos.yml stop postgres
            echo -e "${GREEN}✅ PostgreSQL stopped${NC}"
            echo "To restart: docker-compose -f docker-compose.chaos.yml start postgres"
            ;;
        "CPU Pressure (UBL)")
            echo -e "${YELLOW}Applying CPU pressure to UBL...${NC}"
            docker exec -d ubl-kernel stress-ng --cpu 2 --timeout 60s 2>/dev/null || \
                echo "stress-ng not available in container"
            echo -e "${GREEN}✅ CPU pressure applied (60s)${NC}"
            ;;
        "Memory Pressure (Office)")
            echo -e "${YELLOW}Applying memory pressure to Office... ${NC}"
            docker exec -d office stress-ng --vm 1 --vm-bytes 500M --timeout 60s 2>/dev/null || \
                echo "stress-ng not available in container"
            echo -e "${GREEN}✅ Memory pressure applied (60s)${NC}"
            ;;
        "Remove All Faults")
            echo -e "${GREEN}Removing all faults...${NC}"
            
            # Remove toxiproxy toxics
            curl -X DELETE http://localhost:8474/proxies/ubl/toxics/latency_500 2>/dev/null || true
            curl -X DELETE http://localhost:8474/proxies/ubl/toxics/latency_2000 2>/dev/null || true
            
            # Heal network
            docker network connect chaos-net ubl-kernel 2>/dev/null || true
            
            # Restart services if needed
            docker-compose -f docker-compose.chaos.yml start postgres 2>/dev/null || true
            docker-compose -f docker-compose.chaos.yml up -d ubl-kernel 2>/dev/null || true
            docker-compose -f docker-compose.chaos.yml up -d office 2>/dev/null || true
            
            echo -e "${GREEN}✅ All faults removed${NC}"
            ;;
        "Quit")
            break
            ;;
        *)
            echo "Invalid option $REPLY"
            ;;
    esac
    echo ""
done

echo "Exiting fault injection tool"