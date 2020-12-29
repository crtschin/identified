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

TODO:
- [X] As an admin, I should be able to add/edit/remove new internal users
- [X] As an admin, I should be able to query all internal users
- [X] As a internal user, I should be able to log in and receive a authorization token
- [ ] As an internal user, I should be able to add/edit/remove users
- [ ] As an internal user, I should be able to add/edit/remove roles
- [ ] As an internal user, I should be able to add/edit/remove permissions
- [ ] As an internal user, I should be able to assign roles to user
- [ ] As an internal user, I should be able to assign permissions to users
- [ ] As an internal user, I should be able to assign permissions to roles
- [ ] As an internal user, I should be able to check if a user has a specific permission
- [ ] As an internal user, I should be able to subscribe to a stream of logged in users
  - [X] Keep track of current subscriptions
  - [ ] Publish on check
  - [ ] Publish on add/edit/remove

Endpoints:
- internal: GET/POST/PUT/DELETE, the admin operations
- login: send json object containing email and password for auth token
- user: GET/POST/PUT/DELETE
- permission: GET/POST/PUT/DELETE
- roles: GET/POST/PUT/DELETE
- user/role: GET/POST/PUT/DELETE 
- user/permission: GET/POST/PUT/DELETE
- role/permission: GET/POST/PUT/DELETE
-  ...manage permissions/roles/check authorization