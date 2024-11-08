# Name of the Docker Compose file
COMPOSE_FILE=docker-compose.yml

# Define project name
PROJECT_NAME=orb


all: up

re: down up

fclean: down rm

clean: down

# Bring up the services (detached mode by default)
up:
	docker compose -f $(COMPOSE_FILE) -p $(PROJECT_NAME) up -d

# Bring up the services without detached mode
up-foreground:
	docker compose -f $(COMPOSE_FILE) -p $(PROJECT_NAME) up

# Stop and remove containers, networks, volumes, and images created by up
down:
	docker compose -f $(COMPOSE_FILE) -p $(PROJECT_NAME) down

# Build or rebuild services
build:
	docker compose -f $(COMPOSE_FILE) -p $(PROJECT_NAME) build

# Pull the latest images from the repository
pull:
	docker compose -f $(COMPOSE_FILE) -p $(PROJECT_NAME) pull

# Restart services
restart:
	docker compose -f $(COMPOSE_FILE) -p $(PROJECT_NAME) restart $(service)

# Stop all running services
stop:
	docker compose -f $(COMPOSE_FILE) -p $(PROJECT_NAME) stop

# Start all stopped services
start:
	docker compose -f $(COMPOSE_FILE) -p $(PROJECT_NAME) start

# Remove containers and associated volumes
rm:
	docker compose -f $(COMPOSE_FILE) -p $(PROJECT_NAME) rm -f

# Print logs in follow mode
logs:
	docker compose -f $(COMPOSE_FILE) -p $(PROJECT_NAME) logs -f

.PHONY: all re fclean clean logs
