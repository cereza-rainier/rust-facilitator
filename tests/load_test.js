import http from 'k6/http';
import { check, sleep } from 'k6';
import { Rate, Trend } from 'k6/metrics';

// Custom metrics
const errorRate = new Rate('errors');
const verifyDuration = new Trend('verify_duration');

export const options = {
  stages: [
    { duration: '30s', target: 20 },   // Ramp up to 20 users
    { duration: '1m', target: 50 },    // Ramp up to 50 users
    { duration: '2m', target: 100 },   // Ramp up to 100 users
    { duration: '3m', target: 200 },   // Ramp up to 200 users
    { duration: '2m', target: 200 },   // Stay at 200 users
    { duration: '1m', target: 0 },     // Ramp down to 0 users
  ],
  thresholds: {
    http_req_duration: ['p(95)<500'],  // 95% of requests < 500ms
    http_req_failed: ['rate<0.01'],    // Error rate < 1%
    errors: ['rate<0.01'],
  },
};

const BASE_URL = __ENV.BASE_URL || 'http://localhost:3000';

export default function () {
  // Test 1: Health Check
  const healthRes = http.get(`${BASE_URL}/health`);
  check(healthRes, {
    'health status is 200': (r) => r.status === 200,
    'health response time < 100ms': (r) => r.timings.duration < 100,
  }) || errorRate.add(1);

  // Test 2: Supported Endpoint
  const supportedRes = http.get(`${BASE_URL}/supported`);
  check(supportedRes, {
    'supported status is 200': (r) => r.status === 200,
    'supported has schemes': (r) => {
      try {
        const body = JSON.parse(r.body);
        return body.schemes && body.schemes.length > 0;
      } catch {
        return false;
      }
    },
  }) || errorRate.add(1);

  // Test 3: Metrics Endpoint
  const metricsRes = http.get(`${BASE_URL}/metrics`);
  check(metricsRes, {
    'metrics status is 200': (r) => r.status === 200,
    'metrics has prometheus data': (r) => r.body.includes('x402_'),
  }) || errorRate.add(1);

  // Test 4: Admin Health
  const adminHealthRes = http.get(`${BASE_URL}/admin/health`);
  check(adminHealthRes, {
    'admin health status is 200': (r) => r.status === 200,
  }) || errorRate.add(1);

  sleep(0.1); // 100ms between iterations
}

export function handleSummary(data) {
  return {
    'stdout': textSummary(data, { indent: ' ', enableColors: true }),
    'load-test-results.json': JSON.stringify(data, null, 2),
  };
}


