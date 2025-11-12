# Kubernetes Deployment

Production-ready Kubernetes manifests for x402 Rust Facilitator.

## Prerequisites

- Kubernetes cluster (1.24+)
- kubectl configured
- Docker image pushed to registry

## Quick Start

1. **Create the secret:**
```bash
kubectl create secret generic facilitator-secrets \
  --from-literal=fee_payer_private_key=YOUR_BASE58_PRIVATE_KEY_HERE
```

2. **Apply the manifests:**
```bash
kubectl apply -f configmap.yaml
kubectl apply -f deployment.yaml
kubectl apply -f service.yaml
kubectl apply -f hpa.yaml
```

3. **Verify deployment:**
```bash
kubectl get pods -l app=x402-facilitator
kubectl get svc x402-facilitator
```

4. **Check logs:**
```bash
kubectl logs -l app=x402-facilitator --tail=100 -f
```

## Configuration

### ConfigMap (`configmap.yaml`)

Adjust these values for your environment:
- `solana_rpc_url`: Your Solana RPC endpoint
- `network`: "solana" (mainnet), "solana-devnet", or "solana-testnet"
- `cache_size`: Number of accounts to cache (default: 1000)
- `cache_ttl_seconds`: Cache TTL in seconds (default: 30)
- `enable_rate_limit`: Enable rate limiting (default: "true")
- `rate_limit_per_second`: Requests per second (default: 10)
- `rate_limit_burst_size`: Burst size (default: 20)

### Secrets (`secret.yaml.template`)

**NEVER commit secrets to git!** Use the template to create your secret:

```bash
# Option 1: From literal
kubectl create secret generic facilitator-secrets \
  --from-literal=fee_payer_private_key=YOUR_KEY

# Option 2: From file
echo -n "YOUR_KEY" > /tmp/key.txt
kubectl create secret generic facilitator-secrets \
  --from-file=fee_payer_private_key=/tmp/key.txt
rm /tmp/key.txt
```

## Deployment Features

### High Availability
- **3 replicas** by default
- **Anti-affinity** rules (can be added)
- **Pod disruption budget** (can be added)

### Auto-scaling
- **HPA** scales 3-10 pods based on:
  - CPU utilization (target: 70%)
  - Memory utilization (target: 80%)
- Conservative scale-down (5min stabilization)
- Aggressive scale-up (30s stabilization)

### Health Checks
- **Liveness probe**: Restarts unhealthy pods
- **Readiness probe**: Removes from service when not ready
- **Startup probe**: Allows 50s for initialization

### Resource Limits
- **Requests**: 64Mi RAM, 100m CPU
- **Limits**: 256Mi RAM, 500m CPU

## Services

### External LoadBalancer
```bash
kubectl get svc x402-facilitator
# Access at: http://<EXTERNAL-IP>/
```

### Internal ClusterIP
```bash
kubectl get svc x402-facilitator-internal
# Access at: x402-facilitator-internal:3000 (within cluster)
```

## Monitoring

### Prometheus Metrics
Metrics are exposed at `/metrics`:
```bash
kubectl port-forward svc/x402-facilitator-internal 3000:3000
curl http://localhost:3000/metrics
```

### Health Check
```bash
kubectl port-forward svc/x402-facilitator 8080:80
curl http://localhost:8080/health
curl http://localhost:8080/admin/health
```

## Updating

### Rolling Update
```bash
# Update image
kubectl set image deployment/x402-facilitator \
  facilitator=yourregistry/x402-facilitator:v2.1

# Watch rollout
kubectl rollout status deployment/x402-facilitator
```

### Rollback
```bash
kubectl rollout undo deployment/x402-facilitator
```

## Troubleshooting

### Pod not starting
```bash
kubectl describe pod <pod-name>
kubectl logs <pod-name>
```

### Check configuration
```bash
kubectl get configmap facilitator-config -o yaml
kubectl get secret facilitator-secrets -o yaml
```

### Debug container
```bash
kubectl exec -it <pod-name> -- /bin/sh
```

## Production Recommendations

1. **Use separate namespaces** for dev/staging/prod
2. **Configure Pod Disruption Budget**:
```yaml
apiVersion: policy/v1
kind: PodDisruptionBudget
metadata:
  name: x402-facilitator-pdb
spec:
  minAvailable: 2
  selector:
    matchLabels:
      app: x402-facilitator
```

3. **Add Network Policies** for security
4. **Configure Ingress** instead of LoadBalancer:
```yaml
apiVersion: networking.k8s.io/v1
kind: Ingress
metadata:
  name: x402-facilitator
spec:
  rules:
  - host: facilitator.example.com
    http:
      paths:
      - path: /
        pathType: Prefix
        backend:
          service:
            name: x402-facilitator
            port:
              number: 80
```

5. **Set up monitoring** with Prometheus/Grafana
6. **Configure log aggregation** (ELK, Loki, etc.)
7. **Use managed secrets** (Vault, AWS Secrets Manager, etc.)

## Cleanup

```bash
kubectl delete -f hpa.yaml
kubectl delete -f service.yaml
kubectl delete -f deployment.yaml
kubectl delete -f configmap.yaml
kubectl delete secret facilitator-secrets
```

