import http from 'k6/http';
import { check, sleep } from 'k6';
import { Rate } from 'k6/metrics';
import { SharedArray } from 'k6/data';

const errorRate = new Rate('errors');

export const options = {
  scenarios: {
    message_senders: {
      executor: 'ramping-vus',
      startVUs: 0,
      stages: [
        { duration: '1m', target: 50 },
        { duration: '3m', target: 50 },
        { duration: '1m', target: 0 },
      ],
      gracefulRampDown: '30s',
    },
    job_creators: {
      executor:  'ramping-vus',
      startVUs: 0,
      stages: [
        { duration:  '1m', target: 20 },
        { duration: '3m', target: 20 },
        { duration: '1m', target: 0 },
      ],
      gracefulRampDown: '30s',
    },
  },
  thresholds: {
    'http_req_duration': ['p(95)<1000'],
    'errors': ['rate<0.1'],
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
  const scenario = __ENV.SCENARIO;
  
  if (scenario === 'message_senders') {
    sendMessage(data);
  } else {
    createJob(data);
  }
}

function sendMessage(data) {
  const payload = JSON.stringify({
    conversation_id: data.conversationId,
    content: `Message from VU ${__VU} iteration ${__ITER}`,
    idempotency_key: `idem_msg_${__VU}_${__ITER}`,
  });
  
  const res = http.post(
    `${BASE_URL}/v1/conversations/${data.conversationId}/messages`,
    payload,
    { headers: { 'Content-Type': 'application/json' } }
  );
  
  const success = check(res, {
    'message sent': (r) => r.status === 200,
  });
  
  errorRate.add(!success);
  sleep(1);
}

function createJob(data) {
  const payload = JSON.stringify({
    conversation_id: data.conversationId,
    content: `Create task from VU ${__VU}`,
    idempotency_key: `idem_job_${__VU}_${__ITER}`,
  });
  
  const res = http.post(
    `${BASE_URL}/v1/conversations/${data.conversationId}/messages`,
    payload,
    { headers: { 'Content-Type': 'application/json' } }
  );
  
  const success = check(res, {
    'job created': (r) => r.status === 200,
  });
  
  errorRate.add(!success);
  sleep(3);
}