export async function onRequestPost(context) {
  const { request, env } = context;
  
  try {
    const body = await request.json();
    const eventType = body.type;
    const customerId = body.data.object.customer;

    if (!customerId) {
      return new Response("No customer found", { status: 400 });
    }

    // Сценарий 1: Счет успешно оплачен -> АКТИВИРУЕМ
    if (eventType === "invoice.paid") {
      await env.KEETANET_AUTH.put(customerId, "active");
      return new Response(JSON.stringify({ status: "activated" }), { status: 200 });
    }

    // Сценарий 2: Оплата не прошла или отменена -> БЛОКИРУЕМ
    if (eventType === "invoice.payment_failed" || eventType === "invoice.voided") {
      await env.KEETANET_AUTH.put(customerId, "blocked");
      return new Response(JSON.stringify({ status: "blocked" }), { status: 200 });
    }

    return new Response(JSON.stringify({ received: true }), { status: 200 });
  } catch (err) {
    return new Response(`Webhook Error: ${err.message}`, { status: 400 });
  }
}
