#!/bin/bash

echo "🌟 Universal API Rate Limiter Demo"
echo "=================================="
echo ""

# Check if server is running
if ! curl -s http://127.0.0.1:3000/register > /dev/null; then
    echo "❌ Server is not running!"
    echo "💡 Start it with:"
    echo "   export TARGET_API_URL=\"https://httpbin.org\""
    echo "   export RATE_LIMIT=\"5\""
    echo "   cargo run"
    exit 1
fi

echo "✅ Server is running!"
echo ""

echo "📋 Configuration:"
echo "   Target API: httpbin.org (for testing)"
echo "   Rate Limit: 5 requests per minute"
echo ""

echo "🔑 Step 1: Getting an access token..."
TOKEN_RESPONSE=$(curl -s -X POST http://127.0.0.1:3000/register)
TOKEN=$(echo $TOKEN_RESPONSE | grep -o '"token":"[^"]*"' | cut -d'"' -f4)

if [ -z "$TOKEN" ]; then
    echo "❌ Failed to get token"
    exit 1
fi

echo "✅ Got token: $TOKEN"
echo ""

echo "🚀 Step 2: Making requests through the proxy..."
echo "   Instead of: curl https://httpbin.org/get"
echo "   We call:    curl http://127.0.0.1:3000/api/get"
echo ""

SUCCESS=0
BLOCKED=0

for i in {1..8}; do
    echo -n "Request $i: "
    
    RESPONSE=$(curl -s -w "%{http_code}" -H "Authorization: Bearer $TOKEN" http://127.0.0.1:3000/api/get)
    HTTP_CODE="${RESPONSE: -3}"
    
    if [ "$HTTP_CODE" == "200" ]; then
        SUCCESS=$((SUCCESS + 1))
        echo "✅ SUCCESS (proxied to httpbin.org)"
    elif [ "$HTTP_CODE" == "429" ]; then
        BLOCKED=$((BLOCKED + 1))
        echo "🚫 RATE LIMITED"
    else
        echo "❓ Unexpected: $HTTP_CODE"
    fi
    
    sleep 0.5
done

echo ""
echo "📊 Results:"
echo "   ✅ Successful: $SUCCESS"
echo "   🚫 Rate limited: $BLOCKED"
echo ""

if [ $SUCCESS -le 5 ] && [ $BLOCKED -gt 0 ]; then
    echo "🎉 Demo successful!"
    echo "   Rate limiter correctly allowed ~5 requests and blocked the rest"
else
    echo "⚠️  Unexpected results - check the configuration"
fi

echo ""
echo "💡 In real usage, you would:"
echo "   1. Set TARGET_API_URL to your actual API (OpenAI, Stripe, etc.)"
echo "   2. Set TARGET_API_KEY to your real API key"
echo "   3. Change your app to call http://127.0.0.1:3000/api/* instead"
echo ""
echo "🔒 Benefits:"
echo "   • Your frontend never sees the real API key"
echo "   • You can't accidentally exceed rate limits"
echo "   • Monitor and control API usage per user" 