import http from 'k6/http';
import { check, sleep } from 'k6';
import { Rate } from 'k6/metrics';

const errorRate = new Rate('errors');

export const options = {
  stages:  [
    { duration: '30s', target: 5 },
    { duration: '1m', target: 5 },
    { duration: '30s', target: 20 },
    { duration: '2m', target: 20 },
    { duration: '30s', target: 0 },
  ],
  thresholds: {
    'http_req_duration': ['p(95)<2000', 'p(99)<5000'],
    'errors': ['rate<0.1'],
  },
};

const BASE_URL = __ENV.BASE_URL || 'http://localhost:8080';
const TENANT_ID = 'T.UBL';

export function setup() {
  const bootstrapRes = http.get(`${BASE_URL}/messenger/bootstrap?tenant_id=${TENANT_ID}`);
  const data = JSON.parse(bootstrapRes. body);
  
  return {
    conversationId: data. conversations[0]?.id || 'conv_default',
  };
}

export default function(data) {
  // Send message that triggers job
  const messagePayload = JSON.stringify({
    conversation_id: data.conversationId,
    content: `Create a test report ${Date.now()}`,
    idempotency_key: `idem_${__VU}_${__ITER}`,
  });
  
  const params = {
    headers: {
      'Content-Type': 'application/json',
    },
  };
  
  // Send message
  const messageRes = http.post(
    `${BASE_URL}/v1/conversations/${data.conversationId}/messages`,
    messagePayload,
    params
  );
  
  check(messageRes, {
    'message sent': (r) => r.status === 200,
  });
  
  sleep(2);
  
  // Get timeline to find job
  const timelineRes = http.get(
    `${BASE_URL}/v1/conversations/${data.conversationId}/timeline`
  );
  
  const success = check(timelineRes, {
    'timeline retrieved': (r) => r.status === 200,
    'response time < 1s': (r) => r.timings.duration < 1000,
  });
  
  errorRate.add(!success);
  
  sleep(3);
}