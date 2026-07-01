We will be working on the bodhi mcp discover v2.html page.

In this page, we will be integrating and configuring the MCP connection.

We need some rearrangement of the right side panel.
0. remove Connect, OAuth, 7.4k downloads, as we do not have this information in the API, collapsing/removing this section completely
1. We will have the About Panel, which will have the description. and the meta information like url, transport, supported auth: oauth, key, public

the tools from this is going to go in its own section, and we might not even have this information, so the whole tools tab need to have examples where we do not have this information

we do not have the stats, so the stats section also goes away from the About

2. complete capabilities section goes away, as it is repeat of tools and meta information that is moved to About

3. Connection also goes away as the meta information here moved to about (url, transport), and auth methods configured will be moved as well

5. Metadata also can be removed, sub-sumed in About 

4. Connect stays as is, we have My Instances, and Connect with ... section

the modification here is:
Connect with is only shown for admin, for non-admin, we will have request MCP Server Approval
For admin, we need to move our MCP Server form here
here is the flow/domain for mcp, we have 2 entities- mcp-server and mcp instance
on this MCP Explore page, we have MCP catalog, list of mcp servers that are available to be added and instance created

for a mcp server, you have base url, and transport
in mcp specs, possible values of transport are stdio,sse,http-streamable, we only support http-streamable, so catalog shown here supports http-streamable as one of the transport and that is what we connect to

each mcp-server have many auth mechanisms configured
for auth, we have public (no authentication), oauth, header/query key value
in oauth, either we already know the client id/secret, or we need to manually/offline create client id/secret and provide it, this is oauth+pre-registered client
or we have dynamic client registration, based on convention, using the mcp base url we discover endpoints and register ourselves (Bodhi App) as client dynamic and receive client id/secret and use it, this is oauth+dynamic client registration
or the server supports api token in header or query, so we need to know the header/query keys that needs to be sent

above is only knowing and setting up the auth mechanism - public, oauth: pre-registeered/dynamic-client registration, header/query keys
a mcp-server can have multiple auth mechanism set up, sometimes, multiple auth mechanism of same type, so we have multiple auth mechanisms for a given mcp-server (base url)
so can have oauth-dcr where it has done registration multiple times and received multiple set of client-id/secrets
to keep domain consistency, we by default have a public auth mechanism for all mcp-servers, without having any database entry, so we always show public auth mechanism, for others (oauth, header/key value), you need to have entry with details
so a mcp-server have many auth-mechanism configured falling in 3 types

so we have auth mechanism, then we have actual auth which creates mcp instance, for this, we need the mcp-server created above which has mcp information (base url) and auth mechanism (public, oauth, header/key keys)
public: by default we have public as auth mechanism, user/admin need not configure it, if a mcp server does not have public access, on request it will complain, if it does, well and good
oauth: for oauth flow, dcr or pre-registered does not matter, that is only for auth mechanism, the oauth flow is same for either type, as finally client-id/secret, token url, login url etc. is what matters, in oauth flow, user clicks on connect, and we take him through oauth flow, finally exchanging code for token and storing it against the user, mcp server+auth-mechanism, and then allowing user to chose this mcp instance for request

mcp catalog -1-to-1-> mcp server -1-to-*-> auth-mechanism (public, oauth, header/key keys) -1-to-*-> mcp-instance

we are thinking on this page, either we need ability to trigger the creating of auth-mechanism going to its own dedicated page where information is pre-filled, or allow creating of auth-mechanism itself from the right sidepanel
the second case would work out as follows:
- we do not need to configure public as auth mechanism, as it is always shown on mcp instance form without even configuring it
- we need to have oauth-dcr, oauth-pre-registered, header/query keys supported here
- you select oauth, using the mcp base url, it does auto-discovery and fills in client-id/secret etc. form ready for submit
- if discovery fails, shows the error message dcr failed, at which point form updates to allow user to enter client id, secret (optional), token, login, logout url
- for header/query keys, you have form to allow you to enter keys for header or query, 1 or multiple

given so far, we have not have a form on the right sidepanel, i am not keen to introduce it now, so leaning towards allowing to select from OAuth or Header/Query here, or not even giving those options, just Configure, and it takes to existing MCP Server form with baseurl filled in, remaining interactions happen on that page

also we need to update our existing New Instance form, where the MCP Server dropdown, you can have server configured with multiple auth-mechanism type (oauth, key), and in the Auth Configuration dropdown, you select auth mechanism, including always available public, then on selecting, either you are shown connect for oauth, value for header/query, or nothing for public

So our Conect tab is going to look like:
My Instances: listing down the instances configured, with link to go to playground, or edit or delete
remove Connect With and Primary Action (Manage Instances)

Now, we are looking for appropriate actions in this revamped panel
we have 2 types of MCP Servers, one for which we have configured at least one auth mechanism so have an entry in our db, and one for which we do not have any entry in our db
also note, at each mcp-instance-row, we have enabled/disabled, that enables/disables this mcp app-wide


for mcp-server with no entries in the db:

in future, we also want mechanism for user to request for mcp to be added to app, so for each of the catalog entries, user should be able to request for mcp to be added/configured, and based on his and mcp-server, should be able to see status that he has already requested, and is waiting for approval, or is rejected, this is only for non-admin, and for mcp-servers not configured

admin approval screen is different, and is not part of the scope

when admin comes to a mcp catalog server that is not added, he should be able to one-click be taken to mcp-server form with base url filled in and has to complete remaining auth-mechanism steps

for mcp-server with an entry in db:
a user comes to the mcp-server catalog page, he is able to see the auth-mechanism configured, should be able to click on one of those, and is taken to mcp server instance page with auth preselected, and waiting for user to either submit (auth=public, no more information needed), Connect (oauth), header/query form (header/query keys), saving on submit creates the mcp instance for him
also a user should be able to see previously created mcp instances and is able to go to the playground, or click edit and go to edit form from here, or delete the mcp instance

The My MCPs page in that case becomes very similar to Explore MCPs page, it will only have mcps which have at least an entry in the system
- will have a filter to only show mcp where user have setup auth-mechanism
- the same right sidepanel detail can take him to playground, or delete the mcp server
- also allow to create one selecting the available auth mechanism

so very similar to Explore page

if you have any questions, ask me using AskUserQuestion tool

