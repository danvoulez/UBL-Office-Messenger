import http from 'k6/http';
import { check, sleep } from 'k6';
import { Rate, Trend } from 'k6/metrics';

const errorRate = new Rate('errors');
const memoryLeakDetector = new Trend('response_size');

export const options = {
  stages: [
    { duration: '5m', target: 50 },     // Ramp up
    { duration: '4h', target: 50 },     // Soak for 4 hours
    { duration: '5m', target: 0 },      // Ramp down
  ],
  thresholds: {
    'http_req_duration': ['p(95)<1000'],
    'errors': ['rate<0.05'],
  },
};

const BASE_URL = __ENV.BASE_URL || 'http://localhost:8080';

export function setup() {
  const bootstrapRes = http.get(`${BASE_URL}/messenger/bootstrap? tenant_id=T.UBL`);
  const data = JSON.parse(bootstrapRes.body);
  
  return {
    conversationId: data.conversations[0]?.id || 'conv_default',
  };
}

export default function(data) {
  // Send message
  const payload = JSON.stringify({
    conversation_id: data.conversationId,
    content: `Soak test message ${Date. now()}`,
    idempotency_key: `soak_${__VU}_${__ITER}`,
  });
  
  const res = http.post(
    `${BASE_URL}/v1/conversations/${data.conversationId}/messages`,
    payload,
    { headers: { 'Content-Type': 'application/json' } }
  );
  
  const success = check(res, {
    'status is 200': (r) => r.status === 200,
    'response time < 1s': (r) => r.timings.duration < 1000,
  });
  
  errorRate.add(!success);
  memoryLeakDetector.add(res.body.length);
  
  sleep(1);
}

export function teardown(data) {
  console.log('Soak test complete - check for memory leaks');
}