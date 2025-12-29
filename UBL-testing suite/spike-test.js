import http from 'k6/http';
import { check, sleep } from 'k6';
import { Rate } from 'k6/metrics';

const errorRate = new Rate('errors');

export const options = {
  stages: [
    { duration: '1m', target: 10 },      // Normal load
    { duration: '30s', target: 500 },    // Sudden spike! 
    { duration: '3m', target: 500 },     // Sustained spike
    { duration: '30s', target: 10 },     // Drop back to normal
    { duration: '2m', target: 10 },      // Recovery period
  ],
  thresholds: {
    'http_req_duration': ['p(95)<3000'], // More lenient during spike
    'errors': ['rate<0.3'], // Allow 30% errors during spike
  },
};

const BASE_URL = __ENV.BASE_URL || 'http://localhost:8080';

export function setup() {
  console.log('⚡ Starting spike test - sudden load increase');
  
  const bootstrapRes = http.get(`${BASE_URL}/messenger/bootstrap?tenant_id=T.UBL`);
  const data = JSON.parse(bootstrapRes.body);
  
  return {
    conversationId: data.conversations[0]?.id || 'conv_default',
  };
}

export default function(data) {
  const payload = JSON.stringify({
    conversation_id: data.conversationId,
    content: `Spike test ${__VU}-${__ITER}`,
    idempotency_key: `spike_${__VU}_${__ITER}`,
  });
  
  const res = http.post(
    `${BASE_URL}/v1/conversations/${data. conversationId}/messages`,
    payload,
    {
      headers: { 'Content-Type':  'application/json' },
      timeout: '10s',
    }
  );
  
  const success = check(res, {
    'status ok': (r) => r.status === 200 || r.status === 409,
    'not server error': (r) => r.status < 500,
  });
  
  errorRate.add(!success);
  
  sleep(0.5);
}

export function teardown(data) {
  console.log('\n⚡ Spike test complete');
  console.log('🔍 Check if system: ');
  console.log('   - Handled spike without crashing');
  console.log('   - Recovered to baseline after spike');
  console.log('   - Implemented rate limiting');
  console.log('   - Maintained data integrity');
}