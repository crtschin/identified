Installation instructions 
```
# For Arch minikube (kvm)
# libvirt
sudo pacman -S libvirt
sudo pacman -S qemu virt-manager virt-viewer dnsmasq vde2 bridge-utils openbsd-netcat
sudo pacman -S ebtables iptables

# Start libvirt
sudo systemctl enable libvirtd.service
sudo systemctl start libvirtd.service

# Container stuff
sudo pacman -S docker
sudo pacman -S docker-compose
sudo pacman -S minikube

# Configure minikube to use kvm
minikube config set vm-driver kvm2

# Postgres deps
sudo pacman -S postgresql-libs

# App deps
cargo install diesel_cli --no-default-features --features postgres

source env.sh
make db # runs postgres in a container
```

Usage:
- all: prints all internal users (admin)
- register: new internal users (admin)
- login: send json object containing email and password for auth token
- internal: requires auth token in the header under authorization
  - user
    - /post: register new users
    - /get: get roles/permissions
    - /delete
  - permission
    - /post: create new permissions
    - /get: users/roles
    - /delete
  - roles
    - /post: create new roles
    - /get: users/permissions
    - /delete
  - grant
    - /user_id: roles and permissions
    - /role_id: permission
  - revoke
    - /user_id: roles and permissions
    - /role_id: permission
  - check
    - /user_id: permission
  -  ...manage permissions/roles/check authorization