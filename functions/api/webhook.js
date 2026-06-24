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
      await env.KEETANET_AUTH.put(customerId, "active");
      return Response.json({ status: "activated" });
    }

    if (
      eventType === "invoice.payment_failed" ||
      eventType === "invoice.voided"
    ) {
      await env.KEETANET_AUTH.put(customerId, "blocked");
      return Response.json({ status: "blocked" });
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
