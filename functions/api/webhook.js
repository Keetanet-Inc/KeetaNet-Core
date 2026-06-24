export async function onRequestPost(context) {
  const { request, env } = context;

  try {
    const body = await request.json();

    const eventType = body.type;
    const customerId = body?.data?.object?.customer;

    if (!customerId) {
      return new Response("No customer found", { status: 400 });
    }

    if (eventType === "invoice.paid") {
      await env.KEETANET_AUTH.put(
        customerId,
        JSON.stringify({
          customer: customerId,
          status: "active",
          plan: "pro",

          limits: {
            nodes: 10,
            requests_per_day: 50000
          },

          expires_at: new Date(
            Date.now() + 365 * 24 * 60 * 60 * 1000
          ).toISOString(),

          updated_at: new Date().toISOString()
        })
      );

      return Response.json({
        status: "activated",
        customer: customerId
      });
    }

    if (
      eventType === "invoice.payment_failed" ||
      eventType === "invoice.voided"
    ) {
      await env.KEETANET_AUTH.put(
        customerId,
        JSON.stringify({
          customer: customerId,
          status: "blocked",
          updated_at: new Date().toISOString()
        })
      );

      return Response.json({
        status: "blocked",
        customer: customerId
      });
    }

    return Response.json({
      received: true,
      event: eventType
    });

  } catch (err) {
    return new Response(
      `Webhook Error: ${err.message}`,
      { status: 400 }
    );
  }
}
