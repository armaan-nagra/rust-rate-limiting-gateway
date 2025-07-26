#!/bin/bash

echo "Testing Rate Limiter"
echo "=========================="

# Start the server in the background if not already running
SERVER_URL="http://127.0.0.1:3000"

echo "Step 1: Registering to get a token..."
REGISTER_RESPONSE=$(curl -s -X POST "$SERVER_URL/register")
TOKEN=$(echo $REGISTER_RESPONSE | grep -o '"token":"[^"]*"' | cut -d'"' -f4)

if [ -z "$TOKEN" ]; then
    echo "Failed to get token. Check if the server running?"
    echo "   Start it with: cargo run"
    exit 1
fi

echo "Got token: $TOKEN"
echo ""

echo "Step 2: Making rapid requests to test rate limiting..."
echo "   (Rate limit is set to 60 requests per minute)"
echo ""

SUCCESS_COUNT=0
RATE_LIMITED_COUNT=0

# Make 70 requests to exceed the limit
for i in {1..70}; do
    RESPONSE=$(curl -s -w "%{http_code}" -H "Authorization: Bearer $TOKEN" "$SERVER_URL/api/data")
    HTTP_CODE="${RESPONSE: -3}"
    BODY="${RESPONSE%???}"
    
    if [ "$HTTP_CODE" == "200" ]; then
        SUCCESS_COUNT=$((SUCCESS_COUNT + 1))
        echo "✅ Request $i: SUCCESS (200) - $BODY"
    elif [ "$HTTP_CODE" == "429" ]; then
        RATE_LIMITED_COUNT=$((RATE_LIMITED_COUNT + 1))
        echo "🚫 Request $i: RATE LIMITED (429) - $BODY"
    else
        echo "❓ Request $i: Unexpected response ($HTTP_CODE) - $BODY"
    fi
    
    # Small delay to make output readable
    sleep 0.1
done

echo ""
echo "📊 Results Summary:"
echo "=================="
echo "✅ Successful requests: $SUCCESS_COUNT"
echo "🚫 Rate limited requests: $RATE_LIMITED_COUNT"
echo ""

if [ $SUCCESS_COUNT -le 60 ] && [ $RATE_LIMITED_COUNT -gt 0 ]; then
    echo "🎉 Rate limiter is working correctly!"
    echo "   - Allowed ~60 requests as expected"
    echo "   - Started blocking requests after limit exceeded"
else
    echo "⚠️  Rate limiter may not be working as expected"
    echo "   - Expected: ~60 successful, rest rate limited"
    echo "   - Got: $SUCCESS_COUNT successful, $RATE_LIMITED_COUNT rate limited"
fi 