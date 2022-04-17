const {REST} = require('@discordjs/rest');
const {Routes} = require('discord-api-types/v9');

const commands = [
  {
    name: 'played',
    description: 'Work out how much time someone has spent playing League of Legends recently.',
    type: 1,
    options: [{
        "name": "user",
        "description": "The league of legends username for the user.",
        "required": true,
        "type": 3
    },{
        "name": "days",
        "description": "Over the last how many days.",
        "required": true,
        "type": 4
    }]
  },
];

const rest = new REST({version: '9'}).setToken(process.env.DISCORD_TOKEN);

(async () => {
    try {
      console.log('Started refreshing application (/) commands.');
  
      await rest.put(Routes.applicationCommands(process.env.APP_ID), {
        body: commands,
      });
  
      console.log('Successfully reloaded application (/) commands.');
    } catch (error) {
      console.error(error);
    }
  })();