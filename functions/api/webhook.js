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
          Date.now() + 365 * 24 * 60 * 60 * 1000
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
