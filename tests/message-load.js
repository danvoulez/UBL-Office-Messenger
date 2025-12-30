import http from 'k6/http';
import { check, sleep } from 'k6';
import { Rate } from 'k6/metrics';

const errorRate = new Rate('errors');

export const options = {
  stages: [
    { duration: '30s', target: 10 },   // Ramp up to 10 users
    { duration: '1m', target: 10 },    // Stay at 10 users
    { duration: '30s', target: 50 },   // Ramp up to 50 users
    { duration: '2m', target: 50 },    // Stay at 50 users
    { duration: '30s', target: 100 },  // Ramp up to 100 users
    { duration: '2m', target: 100 },   // Stay at 100 users
    { duration: '30s', target: 0 },    // Ramp down to 0
  ],
  thresholds: {
    'http_req_duration': ['p(95)<500', 'p(99)<1000'],
    'errors': ['rate<0.05'],
  },
};

const BASE_URL = __ENV.BASE_URL || 'http://localhost:8080';
const TENANT_ID = 'T.UBL';

export function setup() {
  // Bootstrap to get conversation
  const bootstrapRes = http.get(`${BASE_URL}/messenger/bootstrap?tenant_id=${TENANT_ID}`);
  const data = JSON.parse(bootstrapRes.body);
  
  return {
    conversationId: data.conversations[0]?.id || 'conv_default',
  };
}

export default function(data) {
  // Send message
  const payload = JSON.stringify({
    conversation_id: data.conversationId,
    content: `Load test message ${Date.now()}`,
    idempotency_key: `idem_${__VU}_${__ITER}`,
  });
  
  const params = {
    headers:  {
      'Content-Type':  'application/json',
    },
  };
  
  const res = http.post(
    `${BASE_URL}/v1/conversations/${data.conversationId}/messages`,
    payload,
    params
  );
  
  const success = check(res, {
    'status is 200': (r) => r.status === 200,
    'response time < 500ms': (r) => r.timings.duration < 500,
    'has message_id': (r) => JSON.parse(r.body).message_id !== undefined,
  });
  
  errorRate.add(!success);
  
  sleep(1);
}

export function teardown(data) {
  console.log('Load test complete');
}