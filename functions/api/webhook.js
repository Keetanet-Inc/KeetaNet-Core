import Stripe from "stripe";

export async function onRequestPost(context) {
  const { request, env } = context;

  const stripe = new Stripe(env.STRIPE_SECRET_KEY);

  try {
    const signature = request.headers.get(
      "stripe-signature"
    );

    if (!signature) {
      return new Response(
        "Missing Stripe signature",
        { status: 401 }
      );
    }

    const payload = await request.text();

    let event;

    try {
      event = await stripe.webhooks.constructEventAsync(
        payload,
        signature,
        env.STRIPE_WEBHOOK_SECRET
      );
    } catch (err) {
      return new Response(
        `Invalid signature: ${err.message}`,
        { status: 401 }
      );
    }

    const customerId =
      event?.data?.object?.customer ||
      event?.data?.object?.id ||
      "test-customer";

    console.log(
      "WEBHOOK",
      event.type,
      customerId
    );

    switch (event.type) {

      case "invoice.paid":
      case "invoice.payment_succeeded":
      case "invoice_payment.paid":
      case "checkout.session.completed":

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
              Date.now() +
              365 * 24 * 60 * 60 * 1000
            ).toISOString(),

            updated_at: new Date().toISOString()
          })
        );

        console.log(
          "LICENSE_ACTIVATED",
          customerId
        );

        return Response.json({
          status: "activated",
          customer: customerId
        });

      case "invoice.payment_failed":
      case "customer.subscription.deleted":

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

      default:
        return Response.json({
          received: true,
          event: event.type
        });
    }

  } catch (err) {
    console.error(err);

    return new Response(
      `Webhook Error: ${err.message}`,
      { status: 400 }
    );
  }
}
