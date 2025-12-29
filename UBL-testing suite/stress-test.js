import http from 'k6/http';
import { check, sleep } from 'k6';
import { Rate, Counter, Trend } from 'k6/metrics';

const errorRate = new Rate('errors');
const requestCount = new Counter('requests');
const successRate = new Rate('success');
const breakingPoint = new Trend('breaking_point_vus');

export const options = {
  stages: [
    { duration: '2m', target: 50 },     // Warm up
    { duration: '5m', target: 100 },    // Stress
    { duration: '5m', target: 200 },    // More stress
    { duration: '5m', target: 500 },    // Heavy stress
    { duration: '5m', target: 1000 },   // Breaking point
    { duration: '10m', target: 1000 },  // Sustained max
    { duration: '5m', target: 0 },      // Recovery
  ],
  thresholds: {
    'http_req_duration': ['p(95)<2000', 'p(99)<5000'],
    'errors': ['rate<0.25'], // Allow 25% errors under extreme stress
    'http_req_failed': ['rate<0.25'],
  },
};

const BASE_URL = __ENV.BASE_URL || 'http://localhost:8080';
const TENANT_ID = 'T.UBL';

export function setup() {
  console.log('🔥 Starting stress test - finding breaking point');
  
  const bootstrapRes = http.get(`${BASE_URL}/messenger/bootstrap?tenant_id=${TENANT_ID}`);
  const data = JSON.parse(bootstrapRes.body);
  
  return {
    conversationId: data.conversations[0]?.id || 'conv_default',
    startTime: Date.now(),
  };
}

export default function(data) {
  const operations = [
    sendMessage,
    queryTimeline,
    healthCheck,
  ];
  
  // Random operation
  const operation = operations[Math. floor(Math.random() * operations.length)];
  operation(data);
  
  requestCount.add(1);
  
  // Shorter sleep under stress
  sleep(0.1);
}

function sendMessage(data) {
  const payload = JSON.stringify({
    conversation_id: data.conversationId,
    content: `Stress test ${__VU}-${__ITER}`,
    idempotency_key: `stress_${__VU}_${__ITER}`,
  });
  
  const res = http.post(
    `${BASE_URL}/v1/conversations/${data.conversationId}/messages`,
    payload,
    {
      headers: { 'Content-Type': 'application/json' },
      timeout: '10s',
    }
  );
  
  const success = check(res, {
    'message sent':  (r) => r.status === 200 || r.status === 409, // 409 = idempotency
  });
  
  errorRate.add(!success);
  successRate.add(success);
  
  if (! success) {
    console.log(`❌ VU ${__VU}:  Message send failed - ${res.status}`);
  }
}

function queryTimeline(data) {
  const res = http.get(
    `${BASE_URL}/v1/conversations/${data.conversationId}/timeline`,
    { timeout: '5s' }
  );
  
  const success = check(res, {
    'timeline queried': (r) => r.status === 200,
  });
  
  errorRate.add(!success);
  successRate.add(success);
}

function healthCheck(data) {
  const res = http.get(`${BASE_URL}/health`, { timeout: '2s' });
  
  const success = check(res, {
    'health check':  (r) => r.status === 200,
  });
  
  errorRate.add(!success);
  successRate.add(success);
}

export function teardown(data) {
  const duration = (Date.now() - data.startTime) / 1000 / 60;
  console.log(`\n🔥 Stress test complete after ${duration. toFixed(1)} minutes`);
  console.log('📊 Check Grafana for performance degradation patterns');
  console.log('🔍 Look for: ');
  console.log('   - Response time increase');
  console.log('   - Error rate spike');
  console.log('   - Resource exhaustion');
  console.log('   - Breaking point (when error rate > 50%)');
}